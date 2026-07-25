package tombi.server

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.application.ReadAction
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.module.ModuleManager
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.intellij.openapi.project.Project
import com.intellij.openapi.roots.ModuleRootManager
import com.intellij.openapi.roots.ProjectRootManager
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServer
import com.intellij.platform.lsp.api.LspServerManager
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.LspServerSupportProvider.LspServerStarter
import com.intellij.platform.lsp.api.lsWidget.LspServerWidgetItem
import com.intellij.util.IncorrectOperationException
import tombi.Icons
import tombi.configurations.TombiConfigurable
import tombi.configurations.tombiConfigurations
import tombi.message
import java.nio.file.Path
import java.time.Duration
import java.util.concurrent.CancellationException
import java.util.concurrent.CompletableFuture
import java.util.concurrent.CompletionException
import java.util.concurrent.ConcurrentHashMap


/**
 * The main entry point of the plugin.
 * 
 * Responsible for starting server instances
 * when TOML files are opened.
 * 
 * @see TombiServerDescriptor
 */
internal class TombiServerSupportProvider : LspServerSupportProvider {
    
    override fun createLspServerWidgetItem(lspServer: LspServer, currentFile: VirtualFile?) =
        LspServerWidgetItem(lspServer, currentFile, Icons._16, TombiConfigurable::class.java)
    
    override fun fileOpened(project: Project, file: VirtualFile, serverStarter: LspServerStarter) {
        if (!file.isTOMLFile || project.isDisposed) {
            return
        }

        if (!registerProject(project) || isInRetryCooldown(project)) {
            return
        }

        val configuredExecutable = tombiConfigurations.executable
        val resolution = resolutions.computeIfAbsent(project) {
            startResolution(project, configuredExecutable)
        }

        resolution.whenComplete { command, error ->
            if (error != null) {
                if (!resolutions.remove(project, resolution) || error is CancellationException) {
                    return@whenComplete
                }
                retryAfterNanos[project] = System.nanoTime() + RETRY_DELAY.toNanos()
                val cause = if (error is CompletionException) error.cause ?: error else error
                LOG.warn("Failed to obtain a Tombi language server", cause)
                notifyDownloadFailure(project, cause)
                return@whenComplete
            }

            ApplicationManager.getApplication().invokeLater {
                if (!project.isDisposed) {
                    LspServerManager.getInstance(project).ensureServerStarted(
                        TombiServerSupportProvider::class.java,
                        TombiServerDescriptor(project, command),
                    )
                }
            }
        }
    }

    companion object {
        private val RETRY_DELAY = Duration.ofMinutes(5)
        private val LOG = Logger.getInstance(TombiServerSupportProvider::class.java)
        private val resolutions = ConcurrentHashMap<Project, CompletableFuture<TombiCommand>>()
        private val retryAfterNanos = ConcurrentHashMap<Project, Long>()
        private val registeredProjects = ConcurrentHashMap.newKeySet<Project>()

        internal fun invalidate(project: Project) {
            resolutions.remove(project)?.cancel(true)
            retryAfterNanos.remove(project)
        }

        /**
         * Resolution runs as a cancellable background task so that the
         * potentially long-running download reports progress to the user.
         */
        private fun startResolution(
            project: Project,
            configuredExecutable: String?,
        ): CompletableFuture<TombiCommand> {
            val resolution = CompletableFuture<TombiCommand>()
            val task = object : Task.Backgroundable(
                project,
                message("progress.resolvingLanguageServer"),
                true,
            ) {
                override fun run(indicator: ProgressIndicator) {
                    resolution.complete(resolve(project, configuredExecutable, indicator))
                }

                override fun onThrowable(error: Throwable) {
                    resolution.completeExceptionally(error)
                }

                override fun onCancel() {
                    resolution.cancel(false)
                }
            }

            // `fileOpened` is invoked from a read action on a background thread, where
            // `ProgressManager.run` would have to `invokeAndWait` and risk a deadlock.
            ApplicationManager.getApplication().invokeLater {
                if (project.isDisposed) {
                    resolution.cancel(false)
                    return@invokeLater
                }
                ProgressManager.getInstance().run(task)
            }

            return resolution
        }

        private fun resolve(
            project: Project,
            configuredExecutable: String?,
            indicator: ProgressIndicator,
        ): TombiCommand {
            // The module model may only be read under a read action.
            val (workspacePaths, sdkHomePaths) = ReadAction.compute<Pair<List<Path>, List<Path>>, RuntimeException> {
                if (project.isDisposed) {
                    emptyList<Path>() to emptyList()
                } else {
                    project.workspacePaths to project.sdkHomePaths
                }
            }

            TombiBinaryResolver.resolveLocal(
                workspacePaths = workspacePaths,
                sdkHomePaths = sdkHomePaths,
                configuredExecutable = configuredExecutable,
            )?.let { return it }

            return TombiCommand(TombiBinaryDownloader.downloadLatestOrCached(indicator = indicator))
        }

        /**
         * @return whether resolution may proceed for [project].
         */
        private fun registerProject(project: Project): Boolean {
            if (!registeredProjects.add(project)) {
                return true
            }

            return try {
                Disposer.register(project) {
                    invalidate(project)
                    registeredProjects.remove(project)
                }
                true
            } catch (error: IncorrectOperationException) {
                registeredProjects.remove(project)
                LOG.debug("Skipped Tombi resolution for an already disposed project", error)
                false
            }
        }

        private fun isInRetryCooldown(project: Project): Boolean {
            val retryAfter = retryAfterNanos[project] ?: return false
            // Subtraction keeps the comparison correct across `nanoTime` wraparound.
            return System.nanoTime() - retryAfter < 0
        }

        private fun notifyDownloadFailure(project: Project, error: Throwable) {
            ApplicationManager.getApplication().invokeLater {
                if (project.isDisposed) {
                    return@invokeLater
                }
                NotificationGroupManager.getInstance()
                    .getNotificationGroup("Tombi")
                    .createNotification(
                        message("notification.languageServerUnavailable.title"),
                        error.message ?: message("notification.languageServerUnavailable.content"),
                        NotificationType.ERROR,
                    )
                    .notify(project)
            }
        }
    }
}


private val Project.workspacePaths: List<Path>
    get() = buildSet {
        path?.let(::add)
        ModuleManager.getInstance(this@workspacePaths).modules.forEach { module ->
            ModuleRootManager.getInstance(module).contentRoots.forEach { root ->
                runCatching { root.toNioPath() }.getOrNull()?.let(::add)
            }
        }
    }.toList()


private val Project.sdkHomePaths: List<Path>
    get() = buildSet {
        ProjectRootManager.getInstance(this@sdkHomePaths).projectSdk?.homePath
            ?.let { runCatching { Path.of(it) }.getOrNull() }
            ?.let(::add)
        ModuleManager.getInstance(this@sdkHomePaths).modules.forEach { module ->
            ModuleRootManager.getInstance(module).sdk?.homePath
                ?.let { runCatching { Path.of(it) }.getOrNull() }
                ?.let(::add)
        }
    }.toList()
