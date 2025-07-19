package tombi.server

import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServerSupportProvider
import com.intellij.platform.lsp.api.LspServerSupportProvider.LspServerStarter
import tombi.configurations.tombiConfigurations


/**
 * The main entry point of the plugin.
 * 
 * Responsible for starting server instances
 * when TOML files are opened.
 * 
 * @see TombiServerDescriptor
 */
internal class TombiServerSupportProvider : LspServerSupportProvider {
    
    override fun fileOpened(project: Project, file: VirtualFile, serverStarter: LspServerStarter) {
        if (file.isTOMLFile) {
            val executable = tombiConfigurations.executable ?: return
            serverStarter.ensureServerStarted(TombiServerDescriptor(project, executable))
        }
    }
    
}
