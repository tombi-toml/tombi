import { gte } from "semver";
import * as vscode from "vscode";
import * as node from "vscode-languageclient/node";
import { bootstrap } from "@/bootstrap";
import * as command from "@/command";
import { log } from "@/logging";
import {
  getStatus,
  getTomlVersion,
  type IgnoreReason,
  updateConfig,
  updateSchema,
} from "@/lsp/client";
import { Server } from "@/lsp/server";
import { clientOptions } from "@/options/client-options";
import { serverOptions } from "@/options/server-options";
import { registerExtensionSchemas } from "@/tomlValidation";
import type { Settings } from "./settings";
export type { Settings };

export const EXTENSION_ID = "tombi";
export const EXTENSION_NAME = "Tombi";
export const SUPPORT_TOML_LANGUAGES = ["toml", "cargoLock"];
export const SUPPORT_TOMBI_CONFIG_FILENAMES = [
  "tombi.toml",
  "pyproject.toml",
  "tombi/config.toml",
];
export const SUPPORT_JSON_LANGUAGES = ["json"];

export class Extension {
  private statusBarItem: vscode.StatusBarItem;
  private lspVersion: string | undefined;

  constructor(
    private context: vscode.ExtensionContext,
    private client: node.LanguageClient,
    private server: Server,
  ) {
    this.statusBarItem = vscode.window.createStatusBarItem(
      vscode.StatusBarAlignment.Left,
    );
    this.context.subscriptions.push(this.statusBarItem);

    this.registerEvents();
    this.registerCommands();
    this.registerExtensionSchemas();
  }

  static async activate(context: vscode.ExtensionContext): Promise<Extension> {
    const settings = vscode.workspace.getConfiguration(
      EXTENSION_ID,
    ) as Settings;

    const tombiBin = await bootstrap(context, settings);

    const server = new Server(tombiBin);
    const client = new node.LanguageClient(
      EXTENSION_ID,
      `${EXTENSION_NAME} Language Server`,
      serverOptions(server.tombiBin.path, settings),
      clientOptions(),
      // biome-ignore lint/complexity/useLiteralKeys: process.env properties require bracket notation
      process.env["__TOMBI_LANGUAGE_SERVER_DEBUG"] !== undefined,
    );

    await client.start;

    const extension = new Extension(context, client, server);

    // Get LSP version
    try {
      extension.lspVersion = await server.showVersion();
      log.info(`Tombi Language Server Version: ${extension.lspVersion}`);
    } catch (error) {
      log.error(`Failed to get LSP version: ${error}`);
    }

    try {
      await client.sendRequest(new node.RequestType("workspace/diagnostic"), {
        previousResultIds: [],
      });
      log.info("Sent `workspace/diagnostic` request");
    } catch (error) {
      log.error(`Failed to send \`workspace/diagnostic\`: ${error}`);
    }

    // NOTE: When VSCode starts, if a TOML document is open in a tab and the focus is not on it,
    //       the Language Server will not start.
    //       Therefore, send the notification to the Language Server for all open TOML documents.
    for (const document of vscode.workspace.textDocuments) {
      await extension.onDidOpenTextDocument(document);
    }

    // Update status bar for initial state
    extension.updateStatusBarItem();

    log.info("extension activated");

    return extension;
  }

  async deactivate(): Promise<void> {
    this.statusBarItem.dispose();
    await this.client.stop();
    log.info("extension deactivated");
  }

  private registerCommands(): void {
    this.context.subscriptions.push(
      vscode.commands.registerCommand(
        `${EXTENSION_ID}.showLanguageServerVersion`,
        async () => command.showLanguageServerVersion(this.server),
      ),
    );
    this.context.subscriptions.push(
      vscode.commands.registerCommand(
        `${EXTENSION_ID}.restartLanguageServer`,
        async () => command.restartLanguageServer(this.client),
      ),
    );
    this.context.subscriptions.push(
      vscode.commands.registerCommand(
        `${EXTENSION_ID}.refreshCache`,
        async () => command.refreshCache(this.client),
      ),
    );
  }

  private registerEvents(): void {
    this.context.subscriptions.push(
      vscode.window.onDidChangeActiveTextEditor(async () => {
        await this.updateStatusBarItem();
      }),
      vscode.workspace.onDidSaveTextDocument(async (document) => {
        await this.onDidSaveTextDocument(document);
        await this.updateStatusBarItem();
      }),
    );
  }

  private registerExtensionSchemas(): void {
    registerExtensionSchemas(this.client);
  }

  private async updateStatusBarItem(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (
      this.lspVersion &&
      editor &&
      SUPPORT_TOML_LANGUAGES.includes(editor.document.languageId)
    ) {
      try {
        let tomlVersion: string;
        let source: string;
        let configPath: string | undefined;
        let ignore: IgnoreReason | undefined;

        if (gte(this.lspVersion, "0.5.1") || this.lspVersion === "0.0.0-dev") {
          // Use getStatus for versions >= 0.5.1
          const response = await this.client.sendRequest(getStatus, {
            uri: editor.document.uri.toString(),
          });
          tomlVersion = response.tomlVersion;
          source = response.source;
          configPath = response.configPath;
          ignore = response.ignore;
        } else {
          // Use getTomlVersion for versions < 0.5.1
          const response = await this.client.sendRequest(getTomlVersion, {
            uri: editor.document.uri.toString(),
          });
          tomlVersion = response.tomlVersion;
          source = response.source;
        }

        let text = `TOML: ${tomlVersion} (${source})`;
        let tooltip = `Tombi: ${this.lspVersion}\nTOML: ${tomlVersion} (${source})\nConfig: ${configPath ?? "default"}`;
        let color: string | vscode.ThemeColor | undefined;

        if (ignore) {
          text = `$(extensions-warning-message) ${text}`;
          tooltip += `\nIgnore: ${ignore.replaceAll("-", " ")}`;
          color = "#D0D0D0";
        }

        this.statusBarItem.text = text;
        this.statusBarItem.color = color;
        this.statusBarItem.backgroundColor = undefined;
        this.statusBarItem.command = `${EXTENSION_ID}.showLanguageServerVersion`;
        this.statusBarItem.tooltip = tooltip;
        this.statusBarItem.show();
      } catch (error) {
        this.statusBarItem.text = "TOML: <unknown>";
        this.statusBarItem.tooltip = `Tombi: ${this.lspVersion}\nTOML: <unknown>\nError: ${error}`;
        this.statusBarItem.color = new vscode.ThemeColor(
          "statusBarItem.errorForeground",
        );
        this.statusBarItem.backgroundColor = new vscode.ThemeColor(
          "statusBarItem.errorBackground",
        );
        this.statusBarItem.show();
      }
    } else {
      this.statusBarItem.hide();
    }
  }

  private async onDidOpenTextDocument(
    document: vscode.TextDocument,
  ): Promise<void> {
    log.info(`onDidOpenTextDocument: ${document.uri.toString()}`);

    if (SUPPORT_TOML_LANGUAGES.includes(document.languageId)) {
      await this.client.sendNotification(
        node.DidOpenTextDocumentNotification.type,
        {
          textDocument: node.TextDocumentItem.create(
            document.uri.toString(),
            document.languageId,
            document.version,
            document.getText(),
          ),
        },
      );
    }
  }

  private async onDidSaveTextDocument(
    document: vscode.TextDocument,
  ): Promise<void> {
    log.info(`onDidSaveTextDocument: ${document.uri.toString()}`);

    if (
      SUPPORT_TOMBI_CONFIG_FILENAMES.some((filename) =>
        document.uri.path.endsWith(filename),
      )
    ) {
      await this.client.sendRequest(updateConfig, {
        uri: document.uri.toString(),
      });
    } else if (SUPPORT_JSON_LANGUAGES.includes(document.languageId)) {
      await this.client.sendRequest(updateSchema, {
        uri: document.uri.toString(),
      });
    }
  }
}
