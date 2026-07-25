package tombi.server

import com.intellij.util.EnvironmentUtil
import java.nio.file.Files
import java.nio.file.Path


internal object TombiBinaryResolver {
    fun resolveLocal(
        projectPath: Path?,
        configuredExecutable: String?,
        environment: Map<String, String> = EnvironmentUtil.getEnvironmentMap(),
        osName: String = System.getProperty("os.name"),
        userHome: String = System.getProperty("user.home"),
    ): String? {
        configuredExecutable
            ?.trim()
            ?.takeIf(String::isNotEmpty)
            ?.let { return expandHome(it, userHome) }

        val isWindows = osName.startsWith("Windows", ignoreCase = true)
        val binaryName = if (isWindows) "tombi.exe" else "tombi"

        if (projectPath != null) {
            val virtualEnvironmentDirectory = if (isWindows) "Scripts" else "bin"
            val projectCandidates = mutableListOf(
                projectPath.resolve(".venv").resolve(virtualEnvironmentDirectory).resolve(binaryName),
                projectPath.resolve("node_modules").resolve(".bin").resolve(binaryName),
            )
            if (isWindows) {
                projectCandidates.addAll(
                    listOf("tombi.cmd", "tombi.ps1").map {
                        projectPath.resolve("node_modules").resolve(".bin").resolve(it)
                    },
                )
            }
            projectCandidates.firstOrNull(Files::isRegularFile)?.let {
                return it.toString()
            }
        }

        return findOnPath(binaryName, environment, isWindows)
    }

    private fun expandHome(path: String, userHome: String): String =
        when {
            path == "~" -> userHome
            path.startsWith("~/") || path.startsWith("~\\") -> userHome + path.drop(1)
            else -> path
        }

    private fun findOnPath(
        binaryName: String,
        environment: Map<String, String>,
        isWindows: Boolean,
    ): String? {
        val pathValue = environment.entries
            .firstOrNull { (key, _) -> key.equals("PATH", ignoreCase = isWindows) }
            ?.value
            ?: return null

        val separator = if (isWindows) ';' else ':'
        val candidateNames = if (isWindows) {
            listOf(binaryName, "tombi.cmd", "tombi.bat")
        } else {
            listOf(binaryName)
        }

        return pathValue
            .split(separator)
            .asSequence()
            .filter(String::isNotBlank)
            .flatMap { directory -> candidateNames.asSequence().map(Path.of(directory)::resolve) }
            .firstOrNull(Files::isRegularFile)
            ?.toString()
    }
}
