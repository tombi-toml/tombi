import * as vscode from "vscode";
import { createInterface } from "node:readline";
import type { Readable } from "node:stream";
import type * as languageclient from "vscode-languageclient/node";
import {
  SUPPORT_TOMBI_CONFIG_FILENAMES,
  SUPPORT_TOML_LANGUAGES,
} from "@/extension";

type LogLevel = "error" | "warn" | "info" | "debug" | "trace";

const TIMESTAMPED_LOG_PREFIX =
  /^(?:\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3} )?\[(ERROR|WARN|WARNING|INFO|DEBUG|TRACE)\] (.*)$/i;
const TOMBI_LOG_PREFIX =
  /^( {2}Error|Warning| {3}Info| {2}Debug| {2}Trace): (.*)$/;

const LOG_LEVEL_BY_NAME = new Map<string, LogLevel>([
  ["error", "error"],
  ["warn", "warn"],
  ["warning", "warn"],
  ["info", "info"],
  ["debug", "debug"],
  ["trace", "trace"],
]);

function writeServerLog(
  outputChannel: vscode.LogOutputChannel,
  level: LogLevel,
  message: string,
): void {
  switch (level) {
    case "error":
      outputChannel.error(message);
      break;
    case "warn":
      outputChannel.warn(message);
      break;
    case "info":
      outputChannel.info(message);
      break;
    case "debug":
      outputChannel.debug(message);
      break;
    case "trace":
      outputChannel.trace(message);
      break;
  }
}

function forwardServerLogs(
  input: Readable,
  outputChannel: vscode.LogOutputChannel,
  fallbackLevel: LogLevel,
): void {
  let level = fallbackLevel;

  createInterface({
    input,
    crlfDelay: Number.POSITIVE_INFINITY,
    terminal: false,
    historySize: 0,
  }).on("line", (line) => {
    const match =
      TIMESTAMPED_LOG_PREFIX.exec(line) ?? TOMBI_LOG_PREFIX.exec(line);
    let message = line;

    if (match) {
      level =
        LOG_LEVEL_BY_NAME.get(match[1]?.trim().toLowerCase() ?? "") ?? level;
      message = match[2] ?? "";
    }

    writeServerLog(outputChannel, level, message);
  });
}

/**
 * `initializationOptions` of the language server.
 *
 * The shape is the contract with the language server:
 * `{ semanticTokens: { strings: { enabled }, comments: { enabled } } }`.
 */
function initializationOptions() {
  const configuration = vscode.workspace.getConfiguration("tombi");
  return {
    semanticTokens: {
      strings: {
        enabled: configuration.get<boolean>(
          "semanticTokens.strings.enabled",
          true,
        ),
      },
      comments: {
        enabled: configuration.get<boolean>(
          "semanticTokens.comments.enabled",
          true,
        ),
      },
    },
  };
}

export function clientOptions(
  workspaceFolder?: vscode.WorkspaceFolder,
): languageclient.LanguageClientOptions {
  const options = {
    diagnosticCollectionName: "tombi",
    documentSelector: SUPPORT_TOML_LANGUAGES.flatMap((language) => [
      { scheme: "file", language },
      { scheme: "untitled", language },
    ]),
    workspaceFolder,
    // NOTE: A function is evaluated on every (re)start of the language server,
    //       so that `Restart Language Server` picks up the changed settings.
    initializationOptions,
    // The Node API requires both callbacks; stdout stays the LSP reader in stdio mode.
    stdioOptions: {
      stdout: (input, outputChannel) =>
        forwardServerLogs(input, outputChannel, "info"),
      stderr: (input, outputChannel) =>
        forwardServerLogs(input, outputChannel, "error"),
    },
    synchronize: {
      // Notify the server about config file changes contained in the workspace.
      fileEvents: SUPPORT_TOMBI_CONFIG_FILENAMES.map((filename) =>
        vscode.workspace.createFileSystemWatcher(`**/${filename}`),
      ),
    },
  } as languageclient.LanguageClientOptions;

  return options;
}
