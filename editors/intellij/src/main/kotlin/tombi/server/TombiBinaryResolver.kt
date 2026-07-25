package tombi.server

import com.intellij.util.EnvironmentUtil
import java.nio.file.Files
import java.nio.file.Path


internal object TombiBinaryResolver {
    private const val MAX_NODE_MODULES_SEARCH_DEPTH = 8

    fun resolveLocal(
        workspacePaths: List<Path>,
        sdkHomePaths: List<Path>,
        configuredExecutable: String?,
        environment: Map<String, String> = EnvironmentUtil.getEnvironmentMap(),
        osName: String = System.getProperty("os.name"),
        userHome: String = System.getProperty("user.home"),
    ): TombiCommand? {
        configuredExecutable
            ?.trim()
            ?.takeIf(String::isNotEmpty)
            ?.let { return TombiCommand(expandHome(it, userHome)) }

        val isWindows = osName.startsWith("Windows", ignoreCase = true)
        val binaryName = if (isWindows) "tombi.exe" else "tombi"

        resolveSdkInstall(sdkHomePaths, binaryName, isWindows)?.let {
            return TombiCommand(it.toString())
        }

        // Checked per workspace, matching the VSCode extension: a virtual environment
        // wins over `node_modules` only within the same workspace, not across them.
        workspacePaths.forEach { workspacePath ->
            resolveVirtualEnvironmentInstall(workspacePath, binaryName, isWindows)?.let {
                return TombiCommand(it.toString())
            }

            resolveNodeModulesInstall(workspacePath, environment, binaryName, isWindows)?.let {
                return it
            }
        }

        return findOnPath(tombiCandidateNames(binaryName, isWindows), environment, isWindows)
            ?.let { TombiCommand(it.toString()) }
    }

    private fun resolveSdkInstall(
        sdkHomePaths: List<Path>,
        binaryName: String,
        isWindows: Boolean,
    ): Path? =
        sdkHomePaths.asSequence()
            .flatMap { sdkHomePath ->
                if (Files.isRegularFile(sdkHomePath)) {
                    sequenceOf(sdkHomePath.parent?.resolve(binaryName))
                } else {
                    sequenceOf(
                        sdkHomePath.resolve(binaryName),
                        sdkHomePath.resolve(if (isWindows) "Scripts" else "bin").resolve(binaryName),
                    )
                }
            }
            .filterNotNull()
            .firstOrNull(Files::isRegularFile)

    private fun resolveVirtualEnvironmentInstall(
        workspacePath: Path,
        binaryName: String,
        isWindows: Boolean,
    ): Path? =
        workspacePath.resolve(".venv")
            .resolve(if (isWindows) "Scripts" else "bin")
            .resolve(binaryName)
            .takeIf(Files::isRegularFile)

    private fun resolveNodeModulesInstall(
        workspacePath: Path,
        environment: Map<String, String>,
        binaryName: String,
        isWindows: Boolean,
    ): TombiCommand? {
        val searchDirectories = nodeModulesSearchDirectories(workspacePath).toList()

        val nodeScript = searchDirectories.asSequence()
            .flatMap { directory ->
                sequenceOf(
                    directory.resolve("node_modules/@tombi-toml/tombi/bin/tombi"),
                    directory.resolve("node_modules/tombi/bin/tombi"),
                )
            }
            .firstOrNull(Files::isRegularFile)

        if (nodeScript != null) {
            findOnPath(
                if (isWindows) listOf("node.exe", "node.cmd") else listOf("node"),
                environment,
                isWindows,
            )?.let { node ->
                return TombiCommand(node.toString(), listOf(nodeScript.toString()))
            }

            if (!isWindows && Files.isExecutable(nodeScript)) {
                return TombiCommand(nodeScript.toString())
            }
        }

        return searchDirectories.asSequence()
            .flatMap { directory ->
                nodeModulesCandidateNames(binaryName, isWindows).asSequence().map { candidateName ->
                    directory.resolve("node_modules/.bin").resolve(candidateName)
                }
            }
            .firstOrNull(Files::isRegularFile)
            ?.let { TombiCommand(it.toString()) }
    }

    private fun nodeModulesSearchDirectories(workspacePath: Path): Sequence<Path> =
        generateSequence(workspacePath) { currentPath ->
            currentPath.parent?.takeIf { it != currentPath }
        }.take(MAX_NODE_MODULES_SEARCH_DEPTH + 1)

    private fun expandHome(path: String, userHome: String): String =
        when {
            path == "~" -> userHome
            path.startsWith("~/") || path.startsWith("~\\") -> userHome + path.drop(1)
            else -> path
        }

    private fun tombiCandidateNames(binaryName: String, isWindows: Boolean): List<String> =
        if (isWindows) {
            listOf(binaryName, "tombi.cmd", "tombi.bat")
        } else {
            listOf(binaryName)
        }

    private fun nodeModulesCandidateNames(binaryName: String, isWindows: Boolean): List<String> =
        if (isWindows) {
            listOf("tombi.cmd", binaryName)
        } else {
            listOf(binaryName)
        }

    private fun findOnPath(
        candidateNames: List<String>,
        environment: Map<String, String>,
        isWindows: Boolean,
    ): Path? {
        val pathValue = environment.entries
            .firstOrNull { (key, _) -> key.equals("PATH", ignoreCase = isWindows) }
            ?.value
            ?: return null

        val separator = if (isWindows) ';' else ':'
        return pathValue
            .split(separator)
            .asSequence()
            .filter(String::isNotBlank)
            .flatMap { directory -> candidateNames.asSequence().map(Path.of(directory)::resolve) }
            .firstOrNull { isRunnable(it, isWindows) }
    }

    /**
     * `PATH` entries have to be runnable to be worth reporting, mirroring the
     * `which` lookup the VSCode extension relies on.
     *
     * The executable bit is only meaningful on POSIX file systems; on Windows
     * runnability is decided by the extension, which [tombiCandidateNames] covers.
     */
    private fun isRunnable(path: Path, isWindows: Boolean): Boolean =
        Files.isRegularFile(path) && (isWindows || Files.isExecutable(path))
}
