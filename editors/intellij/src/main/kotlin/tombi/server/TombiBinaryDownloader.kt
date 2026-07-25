package tombi.server

import com.intellij.openapi.application.PathManager
import com.intellij.openapi.diagnostic.Logger
import com.intellij.util.io.HttpRequests
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream
import org.apache.commons.compress.archivers.zip.ZipArchiveInputStream
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream
import java.io.BufferedInputStream
import java.io.InputStream
import java.io.OutputStream
import java.net.URI
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.AtomicMoveNotSupportedException
import java.nio.file.StandardCopyOption
import java.nio.file.attribute.PosixFilePermission
import java.util.Comparator


internal object TombiBinaryDownloader {
    private const val RELEASES_URL = "https://github.com/tombi-toml/tombi/releases"
    private val versionPattern = Regex("""v?(\d+\.\d+\.\d+)""")

    @Synchronized
    fun downloadLatestOrCached(
        cacheDirectory: Path = PathManager.getSystemDir().resolve("tombi"),
        osName: String = System.getProperty("os.name"),
        architecture: String = System.getProperty("os.arch"),
    ): String {
        val platform = platform(osName, architecture)
        return try {
            downloadLatest(cacheDirectory, platform).toString()
        } catch (error: Exception) {
            newestCachedBinary(cacheDirectory, platform.binaryName)?.toString() ?: throw error
        }
    }

    internal fun platform(osName: String, architecture: String): Platform {
        val targetArchitecture = when (architecture.lowercase()) {
            "aarch64", "arm64" -> "aarch64"
            "x86_64", "amd64", "x64" -> "x86_64"
            else -> error("Unsupported architecture: $architecture")
        }
        val normalizedOsName = osName.lowercase()
        val (targetOs, archiveType) = when {
            normalizedOsName.startsWith("mac") ->
                "apple-darwin" to ArchiveType.TAR_GZ
            normalizedOsName.startsWith("linux") ->
                "unknown-linux-musl" to ArchiveType.TAR_GZ
            normalizedOsName.startsWith("windows") ->
                "pc-windows-msvc" to ArchiveType.ZIP
            else -> error("Unsupported operating system: $osName")
        }

        return Platform(
            target = "$targetArchitecture-$targetOs",
            binaryName = if (archiveType == ArchiveType.ZIP) "tombi.exe" else "tombi",
            archiveType = archiveType,
        )
    }

    internal fun versionFromLatestReleaseUri(uri: URI): String {
        val tag = uri.path.substringAfterLast('/')
        return versionPattern.matchEntire(tag)?.groupValues?.get(1)
            ?: error("Unexpected latest release URL: $uri")
    }

    private fun downloadLatest(
        cacheDirectory: Path,
        platform: Platform,
    ): Path {
        val latestReleaseUri = request("$RELEASES_URL/latest", 30_000).connect { response ->
            URI.create(response.url)
        }
        val version = versionFromLatestReleaseUri(latestReleaseUri)
        val versionDirectory = cacheDirectory.resolve("tombi-$version")
        val binaryPath = versionDirectory.resolve(platform.binaryName)
        if (Files.isRegularFile(binaryPath)) {
            cleanupOldCachedVersions(cacheDirectory, versionDirectory)
            return binaryPath
        }

        Files.createDirectories(versionDirectory)
        val assetName = "tombi-cli-$version-${platform.target}.${platform.archiveType.extension}"
        val assetUri = URI.create("$RELEASES_URL/download/v$version/$assetName")
        val archivePath = Files.createTempFile(versionDirectory, "download-", ".${platform.archiveType.extension}")
        val extractedPath = Files.createTempFile(versionDirectory, "tombi-", ".tmp")

        try {
            request(assetUri.toString(), 120_000).saveToFile(archivePath, null)

            BufferedInputStream(Files.newInputStream(archivePath)).use { input ->
                Files.newOutputStream(extractedPath).use { output ->
                    extractArchive(input, platform.binaryName, platform.archiveType, output)
                }
            }
            makeExecutable(extractedPath)
            moveIntoPlace(extractedPath, binaryPath)
            cleanupOldCachedVersions(cacheDirectory, versionDirectory)
        } finally {
            Files.deleteIfExists(archivePath)
            Files.deleteIfExists(extractedPath)
        }

        return binaryPath
    }

    private fun request(url: String, readTimeoutMillis: Int) =
        HttpRequests.request(url)
            .connectTimeout(15_000)
            .readTimeout(readTimeoutMillis)
            .redirectLimit(10)
            .useProxy(true)
            .productNameAsUserAgent()
            .throwStatusCodeException(true)

    internal fun extractArchive(
        input: InputStream,
        binaryName: String,
        archiveType: ArchiveType,
        output: OutputStream,
    ) {
        when (archiveType) {
            ArchiveType.TAR_GZ -> extractFromTarGz(input, binaryName, output)
            ArchiveType.ZIP -> extractFromZip(input, binaryName, output)
        }
    }

    private fun extractFromTarGz(
        input: InputStream,
        binaryName: String,
        output: OutputStream,
    ) {
        TarArchiveInputStream(GzipCompressorInputStream(input)).use { archive ->
            while (true) {
                val entry = archive.nextEntry ?: break
                if (entry.isFile && Path.of(entry.name).fileName.toString() == binaryName) {
                    archive.copyTo(output)
                    return
                }
            }
        }
        error("$binaryName was not found in the downloaded archive")
    }

    private fun extractFromZip(
        input: InputStream,
        binaryName: String,
        output: OutputStream,
    ) {
        ZipArchiveInputStream(input).use { archive ->
            while (true) {
                val entry = archive.nextEntry ?: break
                if (!entry.isDirectory && Path.of(entry.name).fileName.toString() == binaryName) {
                    archive.copyTo(output)
                    return
                }
            }
        }
        error("$binaryName was not found in the downloaded archive")
    }

    private fun makeExecutable(binaryPath: Path) {
        try {
            val permissions = Files.getPosixFilePermissions(binaryPath)
            permissions.addAll(
                setOf(
                    PosixFilePermission.OWNER_EXECUTE,
                    PosixFilePermission.GROUP_EXECUTE,
                    PosixFilePermission.OTHERS_EXECUTE,
                ),
            )
            Files.setPosixFilePermissions(binaryPath, permissions)
        } catch (_: UnsupportedOperationException) {
            binaryPath.toFile().setExecutable(true, false)
        }
    }

    private fun moveIntoPlace(source: Path, target: Path) {
        try {
            Files.move(
                source,
                target,
                StandardCopyOption.ATOMIC_MOVE,
                StandardCopyOption.REPLACE_EXISTING,
            )
        } catch (_: AtomicMoveNotSupportedException) {
            Files.move(source, target, StandardCopyOption.REPLACE_EXISTING)
        }
    }

    internal fun newestCachedBinary(cacheDirectory: Path, binaryName: String): Path? {
        if (!Files.isDirectory(cacheDirectory)) {
            return null
        }

        return Files.list(cacheDirectory).use { entries ->
            entries
                .map { directory ->
                    val version = versionPattern.matchEntire(directory.fileName.toString().removePrefix("tombi-"))
                        ?.groupValues
                        ?.get(1)
                    val binary = directory.resolve(binaryName).takeIf(Files::isRegularFile)
                    if (version != null && binary != null) {
                        Triple(versionComponents(version), version, binary)
                    } else {
                        null
                    }
                }
                .filter { it != null }
                .map { it!! }
                .max { left, right -> compareVersions(left.first, right.first) }
                .orElse(null)
                ?.third
        }
    }

    internal fun cleanupOldCachedVersions(cacheDirectory: Path, currentVersionDirectory: Path) {
        runCatching {
            if (!Files.isDirectory(cacheDirectory)) {
                return
            }
            Files.list(cacheDirectory).use { entries ->
                entries
                    .filter { it != currentVersionDirectory }
                    .filter(Files::isDirectory)
                    .filter {
                        versionPattern.matches(it.fileName.toString().removePrefix("tombi-"))
                    }
                    .forEach(::deleteRecursively)
            }
        }.onFailure { error ->
            LOG.warn("Failed to remove old cached Tombi binaries", error)
        }
    }

    private fun deleteRecursively(directory: Path) {
        Files.walk(directory).use { entries ->
            entries.sorted(Comparator.reverseOrder()).forEach(Files::deleteIfExists)
        }
    }

    private fun versionComponents(version: String): List<Int> =
        version.split('.').map(String::toInt)

    private fun compareVersions(left: List<Int>, right: List<Int>): Int {
        for (index in 0 until maxOf(left.size, right.size)) {
            val comparison = left.getOrElse(index) { 0 }.compareTo(right.getOrElse(index) { 0 })
            if (comparison != 0) {
                return comparison
            }
        }
        return 0
    }

    internal data class Platform(
        val target: String,
        val binaryName: String,
        val archiveType: ArchiveType,
    )

    internal enum class ArchiveType(val extension: String) {
        TAR_GZ("tar.gz"),
        ZIP("zip"),
    }

    private val LOG = Logger.getInstance(TombiBinaryDownloader::class.java)
}
