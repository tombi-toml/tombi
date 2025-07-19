@file:Suppress("UnstableApiUsage", "unused", "RedundantSamConstructor")

package com.intellij.platform.lsp.tests

import com.intellij.injected.editor.DocumentWindow
import com.intellij.injected.editor.VirtualFileWindow
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspServer
import com.intellij.platform.lsp.api.LspServerManager
import com.intellij.platform.lsp.api.LspServerManagerListener
import com.intellij.platform.lsp.api.LspServerState.ShutdownNormally
import com.intellij.platform.lsp.api.LspServerState.ShutdownUnexpectedly
import com.intellij.testFramework.ExpectedHighlightingData
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.fixtures.CodeInsightTestFixture
import com.intellij.testFramework.fixtures.impl.CodeInsightTestFixtureImpl
import org.junit.Assert
import org.junit.ComparisonFailure
import java.util.concurrent.atomic.AtomicBoolean
import java.util.function.BooleanSupplier

fun waitUntilFileOpenedByLspServer(project: Project, vFile: VirtualFile) {
    val topLevelFile = (vFile as? VirtualFileWindow)?.delegate ?: vFile
    val disposable = Disposer.newDisposable()
    try {
        val fileOpened = AtomicBoolean()
        val serverShutdown = AtomicBoolean()
        LspServerManager.getInstance(project).addLspServerManagerListener(object : LspServerManagerListener {
            override fun serverStateChanged(lspServer: LspServer) {
                if (lspServer.state in arrayOf(ShutdownNormally, ShutdownUnexpectedly)) {
                    serverShutdown.set(true)
                }
            }
            
            override fun fileOpened(lspServer: LspServer, file: VirtualFile) {
                if (file == topLevelFile) fileOpened.set(true)
            }
        }, disposable, sendEventsForExistingServers = true)
        
        PlatformTestUtil.waitWithEventsDispatching("LSP server not initialized in 10 seconds",
                                                   BooleanSupplier { fileOpened.get() || serverShutdown.get() }, 10)
        Assert.assertFalse("LSP server initialization failed", serverShutdown.get())
    }
    finally {
        Disposer.dispose(disposable)
    }
}

/**
 * @apiNote please note that in some cases it isn't enough to call the method once, see related method description
 * @see doCheckExpectedHighlightingData
 */
@JvmOverloads
fun waitForDiagnosticsFromLspServer(project: Project, vFile: VirtualFile, timeout: Int = 30) {
    val topLevelFile = (vFile as? VirtualFileWindow)?.delegate ?: vFile
    val disposable = Disposer.newDisposable()
    try {
        val diagnosticsReceived = AtomicBoolean()
        val serverShutdown = AtomicBoolean()
        LspServerManager.getInstance(project).addLspServerManagerListener(object : LspServerManagerListener {
            override fun serverStateChanged(lspServer: LspServer) {
                if (lspServer.state in arrayOf(ShutdownNormally, ShutdownUnexpectedly)) {
                    serverShutdown.set(true)
                }
            }
            
            override fun diagnosticsReceived(lspServer: LspServer, file: VirtualFile) {
                if (file == topLevelFile) diagnosticsReceived.set(true)
            }
        }, disposable, sendEventsForExistingServers = true)
        
        PlatformTestUtil.waitWithEventsDispatching("Diagnostics from server for file ${vFile.name} not received in $timeout seconds",
                                                   BooleanSupplier { diagnosticsReceived.get() || serverShutdown.get() }, timeout)
        Assert.assertFalse("LSP server initialization failed", serverShutdown.get())
    }
    finally {
        Disposer.dispose(disposable)
    }
}

/**
 * Removes `<error>`/`<warning>` markup from the current file, waits for the `textDocument/publishDiagnostics` notification from the
 * LSP server and checks that the errors/warnings highlighting for the current file match the expected result.
 */
fun CodeInsightTestFixture.checkLspHighlighting() {
    val document = editor.document.let { (it as? DocumentWindow)?.delegate ?: it }
    val data = ExpectedHighlightingData(document, true, true, false)
    data.init() // removes <error>/<warning>/etc. markers
    checkLspHighlightingForData(data)
}

fun CodeInsightTestFixture.checkLspHighlightingForData(data: ExpectedHighlightingData) {
    val vFile = file.virtualFile.let { (it as? VirtualFileWindow)?.delegate ?: it }
    waitForDiagnosticsFromLspServer(project, vFile)
    doCheckExpectedHighlightingData(this as CodeInsightTestFixtureImpl, data)
}

/**
 * LSP servers may send `textDocument/publishDiagnostics` notifications several times for the given file.
 * For example, first: zero problems; second: basic problems (which are quick to calculate);
 * and only the third notification gives all problems for the file.
 * So this function makes up to three attempts to wait for the most up-to-date information from the server and check expected highlighting.
 */
private fun doCheckExpectedHighlightingData(fixture: CodeInsightTestFixtureImpl, data: ExpectedHighlightingData, attemptNumber: Int = 1) {
    val maxAttempts = 3
    
    try {
        fixture.collectAndCheckHighlighting(data)
    }
    catch (cf: ComparisonFailure) {
        if (attemptNumber >= maxAttempts) {
            throw cf
        }
        
        try {
            waitForDiagnosticsFromLspServer(fixture.project, fixture.file.virtualFile, timeout = 5)
        }
        catch (_: AssertionError) {
            // Timed out. The server has probably already sent all the diagnostics and is not going to send any updates.
            throw cf
        }
        
        doCheckExpectedHighlightingData(fixture, data, attemptNumber + 1)
    }
}
