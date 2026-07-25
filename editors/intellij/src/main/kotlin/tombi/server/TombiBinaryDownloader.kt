package tombi.server

import com.intellij.openapi.application.PathManager
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream
import org.apache.commons.compress.archivers.zip.ZipArchiveInputStream
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream
import java.io.BufferedInputStream
import java.net.URI
import java.net.http.HttpClient
import java.net.http.HttpRequest
import java.net.http.HttpResponse
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.AtomicMoveNotSupportedException
import java.nio.file.StandardCopyOption
import java.nio.file.attribute.PosixFilePermission
import java.time.Duration


internal object TombiBinaryDownloader {
    private const val RELEASES_URL = "https://github.com/tombi-toml/tombi/releases"
    private val versionPattern = Regex("""v?(\d+\.\d+\.\d+)""")

    @Synchronized
    fun downloadLatestOrCached(
        cacheDirectory: Path = PathManager.getSystemDir().resolve("tombi"),
        client: HttpClient = HttpClient.newBuilder()
            .connectTimeout(Duration.ofSeconds(15))
            .followRedirects(HttpClient.Redirect.ALWAYS)
            .build(),
        osName: String = System.getProperty("os.name"),
        architecture: String = System.getProperty("os.arch"),
    ): String {
        return try {
            downloadLatest(cacheDirectory, client, platform(osName, architecture)).toString()
        } catch (error: Exception) {
            newestCachedBinary(cacheDirectory)?.toString() ?: throw error
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
        client: HttpClient,
        platform: Platform,
    ): Path {
        val latestRequest = HttpRequest.newBuilder(URI.create("$RELEASES_URL/latest"))
            .timeout(Duration.ofSeconds(30))
            .GET()
            .build()
        val latestResponse = client.send(latestRequest, HttpResponse.BodyHandlers.discarding())
        requireSuccess(latestResponse.statusCode(), latestResponse.uri())

        val version = versionFromLatestReleaseUri(latestResponse.uri())
        val versionDirectory = cacheDirectory.resolve("tombi-$version")
        val binaryPath = versionDirectory.resolve(platform.binaryName)
        if (Files.isRegularFile(binaryPath)) {
            return binaryPath
        }

        Files.createDirectories(versionDirectory)
        val assetName = "tombi-cli-$version-${platform.target}.${platform.archiveType.extension}"
        val assetUri = URI.create("$RELEASES_URL/download/v$version/$assetName")
        val archivePath = Files.createTempFile(versionDirectory, "download-", ".${platform.archiveType.extension}")
        val extractedPath = Files.createTempFile(versionDirectory, "tombi-", ".tmp")

        try {
            val assetRequest = HttpRequest.newBuilder(assetUri)
                .timeout(Duration.ofMinutes(2))
                .GET()
                .build()
            val assetResponse = client.send(assetRequest, HttpResponse.BodyHandlers.ofFile(archivePath))
            requireSuccess(assetResponse.statusCode(), assetResponse.uri())

            BufferedInputStream(Files.newInputStream(archivePath)).use { input ->
                Files.newOutputStream(extractedPath).use { output ->
                    when (platform.archiveType) {
                        ArchiveType.TAR_GZ ->
                            extractFromTarGz(input, platform.binaryName, output)
                        ArchiveType.ZIP ->
                            extractFromZip(input, platform.binaryName, output)
                    }
                }
            }
            makeExecutable(extractedPath)
            moveIntoPlace(extractedPath, binaryPath)
        } finally {
            Files.deleteIfExists(archivePath)
            Files.deleteIfExists(extractedPath)
        }

        return binaryPath
    }

    private fun extractFromTarGz(
        input: java.io.InputStream,
        binaryName: String,
        output: java.io.OutputStream,
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
        input: java.io.InputStream,
        binaryName: String,
        output: java.io.OutputStream,
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

    private fun newestCachedBinary(cacheDirectory: Path): Path? {
        if (!Files.isDirectory(cacheDirectory)) {
            return null
        }

        return Files.list(cacheDirectory).use { entries ->
            entries
                .map { directory ->
                    val version = versionPattern.matchEntire(directory.fileName.toString().removePrefix("tombi-"))
                        ?.groupValues
                        ?.get(1)
                    val binary = listOf("tombi", "tombi.exe")
                        .map(directory::resolve)
                        .firstOrNull(Files::isRegularFile)
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

    private fun requireSuccess(statusCode: Int, uri: URI) {
        check(statusCode in 200..299) {
            "Request to $uri failed with HTTP $statusCode"
        }
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
}
