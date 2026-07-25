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
            "/home/test/bin/tombi",
            TombiBinaryResolver.resolveLocal(
                projectPath = null,
                configuredExecutable = "~/bin/tombi",
                environment = emptyMap(),
                osName = "Linux",
                userHome = "/home/test",
            ),
        )
    }

    @Test
    fun `project virtual environment takes priority over node modules and path`() {
        val project = Files.createTempDirectory("tombi-project")
        val venvTombi = project.resolve(".venv/bin/tombi")
        val nodeModulesTombi = project.resolve("node_modules/.bin/tombi")
        val pathDirectory = Files.createTempDirectory("tombi-path")
        val pathTombi = pathDirectory.resolve("tombi")
        venvTombi.parent.createDirectories()
        nodeModulesTombi.parent.createDirectories()
        venvTombi.createFile()
        nodeModulesTombi.createFile()
        pathTombi.createFile()

        assertEquals(
            venvTombi.toString(),
            TombiBinaryResolver.resolveLocal(
                projectPath = project,
                configuredExecutable = null,
                environment = mapOf("PATH" to pathDirectory.toString()),
                osName = "Linux",
            ),
        )
    }

    @Test
    fun `node modules takes priority over path`() {
        val project = Files.createTempDirectory("tombi-project")
        val nodeModulesTombi = project.resolve("node_modules/.bin/tombi")
        val pathDirectory = Files.createTempDirectory("tombi-path")
        nodeModulesTombi.parent.createDirectories()
        nodeModulesTombi.createFile()
        pathDirectory.resolve("tombi").createFile()

        assertEquals(
            nodeModulesTombi.toString(),
            TombiBinaryResolver.resolveLocal(
                projectPath = project,
                configuredExecutable = null,
                environment = mapOf("PATH" to pathDirectory.toString()),
                osName = "Linux",
            ),
        )
    }

    @Test
    fun `returns path installation when no project installation exists`() {
        val pathDirectory = Files.createTempDirectory("tombi-path")
        val pathTombi = pathDirectory.resolve("tombi").createFile()

        assertEquals(
            pathTombi.toString(),
            TombiBinaryResolver.resolveLocal(
                projectPath = Files.createTempDirectory("tombi-project"),
                configuredExecutable = null,
                environment = mapOf("PATH" to pathDirectory.toString()),
                osName = "Linux",
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
            nodeModulesTombi.toString(),
            TombiBinaryResolver.resolveLocal(
                projectPath = project,
                configuredExecutable = null,
                environment = emptyMap(),
                osName = "Windows 11",
            ),
        )
    }

    @Test
    fun `returns null when no local installation exists`() {
        assertNull(
            TombiBinaryResolver.resolveLocal(
                projectPath = Files.createTempDirectory("tombi-project"),
                configuredExecutable = null,
                environment = emptyMap(),
                osName = "Linux",
            ),
        )
    }
}
