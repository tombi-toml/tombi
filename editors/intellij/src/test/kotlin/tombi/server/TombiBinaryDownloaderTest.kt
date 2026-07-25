package tombi.server

import java.net.URI
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
}
