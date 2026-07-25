package tombi.server

import org.apache.commons.compress.archivers.tar.TarArchiveEntry
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream
import org.apache.commons.compress.archivers.zip.ZipArchiveEntry
import org.apache.commons.compress.archivers.zip.ZipArchiveOutputStream
import org.apache.commons.compress.compressors.gzip.GzipCompressorOutputStream
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.net.URI
import java.nio.file.Files
import java.util.Comparator
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith


class TombiBinaryDownloaderTest {
    @Test
    fun `resolves supported release targets`() {
        assertEquals(
            TombiBinaryDownloader.Platform(
                target = "aarch64-apple-darwin",
                binaryName = "tombi",
                archiveType = TombiBinaryDownloader.ArchiveType.TAR_GZ,
            ),
            TombiBinaryDownloader.platform("Mac OS X", "arm64"),
        )
        assertEquals(
            TombiBinaryDownloader.Platform(
                target = "x86_64-unknown-linux-musl",
                binaryName = "tombi",
                archiveType = TombiBinaryDownloader.ArchiveType.TAR_GZ,
            ),
            TombiBinaryDownloader.platform("Linux", "amd64"),
        )
        assertEquals(
            TombiBinaryDownloader.Platform(
                target = "aarch64-pc-windows-msvc",
                binaryName = "tombi.exe",
                archiveType = TombiBinaryDownloader.ArchiveType.ZIP,
            ),
            TombiBinaryDownloader.platform("Windows 11", "aarch64"),
        )
    }

    @Test
    fun `extracts version from latest release redirect`() {
        assertEquals(
            "1.2.4",
            TombiBinaryDownloader.versionFromLatestReleaseUri(
                URI.create("https://github.com/tombi-toml/tombi/releases/tag/v1.2.4"),
            ),
        )
    }

    @Test
    fun `rejects unsupported platform`() {
        assertFailsWith<IllegalStateException> {
            TombiBinaryDownloader.platform("FreeBSD", "x86_64")
        }
    }

    @Test
    fun `extracts tombi from tar gzip archive`() {
        val binary = "tar binary".toByteArray()
        val archive = ByteArrayOutputStream().also { bytes ->
            GzipCompressorOutputStream(bytes).use { gzip ->
                TarArchiveOutputStream(gzip).use { tar ->
                    val entry = TarArchiveEntry("tombi-cli/tombi").apply {
                        size = binary.size.toLong()
                    }
                    tar.putArchiveEntry(entry)
                    tar.write(binary)
                    tar.closeArchiveEntry()
                }
            }
        }.toByteArray()
        val extracted = ByteArrayOutputStream()

        TombiBinaryDownloader.extractArchive(
            ByteArrayInputStream(archive),
            "tombi",
            TombiBinaryDownloader.ArchiveType.TAR_GZ,
            extracted,
        )

        assertEquals("tar binary", extracted.toString())
    }

    @Test
    fun `extracts tombi from zip archive`() {
        val archive = ByteArrayOutputStream().also { bytes ->
            ZipArchiveOutputStream(bytes).use { zip ->
                zip.putArchiveEntry(ZipArchiveEntry("tombi-cli/tombi.exe"))
                zip.write("zip binary".toByteArray())
                zip.closeArchiveEntry()
            }
        }.toByteArray()
        val extracted = ByteArrayOutputStream()

        TombiBinaryDownloader.extractArchive(
            ByteArrayInputStream(archive),
            "tombi.exe",
            TombiBinaryDownloader.ArchiveType.ZIP,
            extracted,
        )

        assertEquals("zip binary", extracted.toString())
    }

    @Test
    fun `uses newest compatible cached binary`() {
        val cacheDirectory = Files.createTempDirectory("tombi-downloader-test")
        try {
            val oldBinary = cacheDirectory.resolve("tombi-1.9.0/tombi")
            val newestBinary = cacheDirectory.resolve("tombi-1.10.0/tombi")
            val windowsBinary = cacheDirectory.resolve("tombi-2.0.0/tombi.exe")
            listOf(oldBinary, newestBinary, windowsBinary).forEach { binary ->
                Files.createDirectories(binary.parent)
                Files.createFile(binary)
            }
            Files.createDirectories(cacheDirectory.resolve("not-a-version"))

            assertEquals(
                newestBinary,
                TombiBinaryDownloader.newestCachedBinary(cacheDirectory, "tombi"),
            )
            assertEquals(
                windowsBinary,
                TombiBinaryDownloader.newestCachedBinary(cacheDirectory, "tombi.exe"),
            )
        } finally {
            Files.walk(cacheDirectory).use { paths ->
                paths.sorted(Comparator.reverseOrder()).forEach(Files::deleteIfExists)
            }
        }
    }

    @Test
    fun `removes old version caches after a successful resolution`() {
        val cacheDirectory = Files.createTempDirectory("tombi-downloader-test")
        try {
            val oldVersion = cacheDirectory.resolve("tombi-1.9.0")
            val currentVersion = cacheDirectory.resolve("tombi-1.10.0")
            val unrelated = cacheDirectory.resolve("downloads")
            listOf(oldVersion, currentVersion, unrelated).forEach(Files::createDirectories)
            Files.createFile(oldVersion.resolve("tombi"))
            Files.createFile(currentVersion.resolve("tombi"))

            TombiBinaryDownloader.cleanupOldCachedVersions(cacheDirectory, currentVersion)

            assertEquals(false, Files.exists(oldVersion))
            assertEquals(true, Files.exists(currentVersion))
            assertEquals(true, Files.exists(unrelated))
        } finally {
            Files.walk(cacheDirectory).use { paths ->
                paths.sorted(Comparator.reverseOrder()).forEach(Files::deleteIfExists)
            }
        }
    }
}
