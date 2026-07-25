package tombi.server

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
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
import java.nio.file.Files
import java.nio.file.Path
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

        val configuredExecutable = tombiConfigurations.executable
        TombiBinaryResolver.resolveLocal(project.path, configuredExecutable)?.let { executable ->
            serverStarter.ensureServerStarted(TombiServerDescriptor(project, executable))
            return
        }

        managedExecutable
            ?.takeIf { Files.isRegularFile(Path.of(it)) }
            ?.let { executable ->
                serverStarter.ensureServerStarted(TombiServerDescriptor(project, executable))
                return
            }

        val resolution = managedResolutions.computeIfAbsent(project) {
            CompletableFuture.supplyAsync(
                { TombiBinaryDownloader.downloadLatestOrCached() },
                AppExecutorUtil.getAppExecutorService(),
            )
        }

        resolution.whenComplete { executable, error ->
            managedResolutions.remove(project, resolution)

            if (error != null) {
                LOG.warn("Failed to obtain a Tombi language server", error)
                return@whenComplete
            }

            managedExecutable = executable
            ApplicationManager.getApplication().invokeLater {
                if (!project.isDisposed) {
                    LspServerManager.getInstance(project).ensureServerStarted(
                        TombiServerSupportProvider::class.java,
                        TombiServerDescriptor(project, executable),
                    )
                }
            }
        }
    }

    companion object {
        private val LOG = Logger.getInstance(TombiServerSupportProvider::class.java)
        private val managedResolutions = ConcurrentHashMap<Project, CompletableFuture<String>>()
        @Volatile
        private var managedExecutable: String? = null
    }
}
