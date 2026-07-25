package tombi.server

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.module.ModuleManager
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
import com.intellij.util.concurrency.AppExecutorUtil
import tombi.Icons
import tombi.configurations.TombiConfigurable
import tombi.configurations.tombiConfigurations
import tombi.message
import java.nio.file.Path
import java.time.Duration
import java.util.concurrent.CompletableFuture
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
        if (!file.isTOMLFile) {
            return
        }

        registerProject(project)
        if ((retryAfterNanos[project] ?: 0) > System.nanoTime()) {
            return
        }

        val configuredExecutable = tombiConfigurations.executable
        val workspacePaths = project.workspacePaths
        val sdkHomePaths = project.sdkHomePaths

        val resolution = resolutions.computeIfAbsent(project) {
            CompletableFuture.supplyAsync(
                {
                    TombiBinaryResolver.resolveLocal(
                        workspacePaths = workspacePaths,
                        sdkHomePaths = sdkHomePaths,
                        configuredExecutable = configuredExecutable,
                    ) ?: TombiCommand(TombiBinaryDownloader.downloadLatestOrCached())
                },
                AppExecutorUtil.getAppExecutorService(),
            )
        }

        resolution.whenComplete { command, error ->
            if (error != null) {
                if (resolutions.remove(project, resolution)) {
                    retryAfterNanos[project] = System.nanoTime() + RETRY_DELAY.toNanos()
                    val cause = error.cause ?: error
                    LOG.warn("Failed to obtain a Tombi language server", cause)
                    notifyDownloadFailure(project, cause)
                }
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

        private fun registerProject(project: Project) {
            if (registeredProjects.add(project)) {
                Disposer.register(project) {
                    invalidate(project)
                    registeredProjects.remove(project)
                }
            }
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
                add(root.toNioPath())
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
