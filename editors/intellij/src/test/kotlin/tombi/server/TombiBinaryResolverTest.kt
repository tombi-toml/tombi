package tombi.server

import java.nio.file.Files
import kotlin.io.path.createDirectories
import kotlin.io.path.createFile
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull


class TombiBinaryResolverTest {
    @Test
    fun `configured executable takes priority and expands home`() {
        assertEquals(
            TombiCommand("/home/test/bin/tombi"),
            resolveLocal(
                configuredExecutable = "~/bin/tombi",
                userHome = "/home/test",
            ),
        )
    }

    @Test
    fun `selected SDK environment takes priority over project installations`() {
        val sdk = Files.createTempDirectory("tombi-sdk")
        val sdkPython = sdk.resolve("bin/python")
        val sdkTombi = sdk.resolve("bin/tombi")
        val project = Files.createTempDirectory("tombi-project")
        sdkPython.parent.createDirectories()
        sdkPython.createFile()
        sdkTombi.createFile()
        project.resolve(".venv/bin").createDirectories()
        project.resolve(".venv/bin/tombi").createFile()

        assertEquals(
            TombiCommand(sdkTombi.toString()),
            resolveLocal(
                workspacePaths = listOf(project),
                sdkHomePaths = listOf(sdkPython),
            ),
        )
    }

    @Test
    fun `project virtual environment takes priority over node modules and path`() {
        val project = Files.createTempDirectory("tombi-project")
        val venvTombi = project.resolve(".venv/bin/tombi")
        val nodeModulesTombi = project.resolve("node_modules/.bin/tombi")
        val pathDirectory = Files.createTempDirectory("tombi-path")
        venvTombi.parent.createDirectories()
        nodeModulesTombi.parent.createDirectories()
        venvTombi.createFile()
        nodeModulesTombi.createFile()
        pathDirectory.resolve("tombi").createFile()

        assertEquals(
            TombiCommand(venvTombi.toString()),
            resolveLocal(
                workspacePaths = listOf(project),
                environment = mapOf("PATH" to pathDirectory.toString()),
            ),
        )
    }

    @Test
    fun `finds npm package in parent directory and runs it with node`() {
        val repository = Files.createTempDirectory("tombi-repository")
        val project = repository.resolve("packages/example").createDirectories()
        val nodeScript = repository.resolve("node_modules/@tombi-toml/tombi/bin/tombi")
        val nodeDirectory = Files.createTempDirectory("node-path")
        val node = nodeDirectory.resolve("node")
        nodeScript.parent.createDirectories()
        nodeScript.createFile()
        node.createFile()

        assertEquals(
            TombiCommand(node.toString(), listOf(nodeScript.toString())),
            resolveLocal(
                workspacePaths = listOf(project),
                environment = mapOf("PATH" to nodeDirectory.toString()),
            ),
        )
    }

    @Test
    fun `finds hoisted node modules shim before path`() {
        val repository = Files.createTempDirectory("tombi-repository")
        val project = repository.resolve("packages/example").createDirectories()
        val nodeModulesTombi = repository.resolve("node_modules/.bin/tombi")
        val pathDirectory = Files.createTempDirectory("tombi-path")
        nodeModulesTombi.parent.createDirectories()
        nodeModulesTombi.createFile()
        pathDirectory.resolve("tombi").createFile()

        assertEquals(
            TombiCommand(nodeModulesTombi.toString()),
            resolveLocal(
                workspacePaths = listOf(project),
                environment = mapOf("PATH" to pathDirectory.toString()),
            ),
        )
    }

    @Test
    fun `searches every workspace content root`() {
        val firstWorkspace = Files.createTempDirectory("tombi-workspace")
        val secondWorkspace = Files.createTempDirectory("tombi-workspace")
        val nodeModulesTombi = secondWorkspace.resolve("node_modules/.bin/tombi")
        nodeModulesTombi.parent.createDirectories()
        nodeModulesTombi.createFile()

        assertEquals(
            TombiCommand(nodeModulesTombi.toString()),
            resolveLocal(workspacePaths = listOf(firstWorkspace, secondWorkspace)),
        )
    }

    @Test
    fun `returns path installation when no project installation exists`() {
        val pathDirectory = Files.createTempDirectory("tombi-path")
        val pathTombi = pathDirectory.resolve("tombi").createFile()

        assertEquals(
            TombiCommand(pathTombi.toString()),
            resolveLocal(
                workspacePaths = listOf(Files.createTempDirectory("tombi-project")),
                environment = mapOf("PATH" to pathDirectory.toString()),
            ),
        )
    }

    @Test
    fun `finds node modules command shim on Windows`() {
        val project = Files.createTempDirectory("tombi-project")
        val nodeModulesTombi = project.resolve("node_modules/.bin/tombi.cmd")
        nodeModulesTombi.parent.createDirectories()
        nodeModulesTombi.createFile()

        assertEquals(
            TombiCommand(nodeModulesTombi.toString()),
            resolveLocal(
                workspacePaths = listOf(project),
                osName = "Windows 11",
            ),
        )
    }

    @Test
    fun `does not use PowerShell shim on Windows`() {
        val project = Files.createTempDirectory("tombi-project")
        val powerShellShim = project.resolve("node_modules/.bin/tombi.ps1")
        powerShellShim.parent.createDirectories()
        powerShellShim.createFile()

        assertNull(
            resolveLocal(
                workspacePaths = listOf(project),
                osName = "Windows 11",
            ),
        )
    }

    @Test
    fun `does not search node modules beyond the VSCode depth limit`() {
        val repository = Files.createTempDirectory("tombi-repository")
        val project = (1..9).fold(repository) { path, index ->
            path.resolve("level-$index").createDirectories()
        }
        val nodeModulesTombi = repository.resolve("node_modules/.bin/tombi")
        nodeModulesTombi.parent.createDirectories()
        nodeModulesTombi.createFile()

        assertNull(resolveLocal(workspacePaths = listOf(project)))
    }

    @Test
    fun `returns null when no local installation exists`() {
        assertNull(
            resolveLocal(
                workspacePaths = listOf(Files.createTempDirectory("tombi-project")),
            ),
        )
    }

    private fun resolveLocal(
        workspacePaths: List<java.nio.file.Path> = emptyList(),
        sdkHomePaths: List<java.nio.file.Path> = emptyList(),
        configuredExecutable: String? = null,
        environment: Map<String, String> = emptyMap(),
        osName: String = "Linux",
        userHome: String = "/home/test",
    ) = TombiBinaryResolver.resolveLocal(
        workspacePaths = workspacePaths,
        sdkHomePaths = sdkHomePaths,
        configuredExecutable = configuredExecutable,
        environment = environment,
        osName = osName,
        userHome = userHome,
    )
}
