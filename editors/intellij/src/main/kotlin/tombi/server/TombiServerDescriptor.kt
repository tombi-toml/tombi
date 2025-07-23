package tombi.server

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.guessProjectDir
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.ProjectWideLspServerDescriptor
import org.eclipse.lsp4j.ClientCapabilities
import tombi.message
import java.nio.file.Path


// https://github.com/JetBrains/intellij-community/blob/0c6f5f52/plugins/toml/core/src/main/resources/intellij.toml.core.xml#L8
/**
 * A list of TOML files that
 * do not use the extension `.toml`.
 */
private val knownTOMLFiles = listOf(
    "Cargo.lock", "Gopkg.lock", "Pipfile",
    "pdm.lock", "poetry.lock", "uv.lock"
)


internal val VirtualFile.isTOMLFile: Boolean
    get() = extension?.lowercase() == "toml" || name in knownTOMLFiles


/**
 * The directory in which the project resides.
 *
 * Only an approximation, since a project
 * may include multiple directories.
 */
internal val Project.path: Path?
    get() = guessProjectDir()?.toNioPath() ?: basePath?.let { Path.of(it) }


/**
 * "Describe" the language server.
 */
internal class TombiServerDescriptor(project: Project, private val executable: String) :
    ProjectWideLspServerDescriptor(project, PRESENTABLE_NAME) {

    /**
     * The client capabilities to be sent in `initialize`.
     *
     * Prior to 2025.1.2, only `/publishDiagnostics` is supported.
     * 2025.1.2 added support for `/diagnostic`, but disables pulling by default.
     * 2025.2 is thus the first version that support pull semantics out-of-the-box.
     *
     * `textDocument.diagnostic` is set to `null` here
     * to avoid the hassle of dealing with customizations.
     */
    override val clientCapabilities: ClientCapabilities
        get() = super.clientCapabilities.apply {
            textDocument.apply {
                diagnostic = null
            }
        }

    override fun isSupportedFile(file: VirtualFile) =
        file.isTOMLFile

    override fun createCommandLine() = GeneralCommandLine().apply {
        withWorkingDirectory(project.path)
        withCharset(Charsets.UTF_8)

        withExePath(executable)
        addParameter("lsp")
    }

    companion object {
        /**
         * The name of the server,
         * to be used in user-facing messages.
         */
        private val PRESENTABLE_NAME = message("languageServer.presentableName")
    }

}
