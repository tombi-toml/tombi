import * as vscode from "vscode";
import * as node from "vscode-languageclient/node";
import { clientOptions } from "@/options/client-options";
import { serverOptions } from "@/options/server-options";
import { Server } from "@/lsp/server";
import type { Settings } from "./settings";
import * as command from "@/command";
import { bootstrap } from "@/bootstrap";
import { log } from "@/logging";
import { getTomlVersion, updateConfig, updateSchema } from "@/lsp/client";
import { registerExtensionSchemas } from "@/tomlValidation";
export type { Settings };

export const EXTENTION_ID = "tombi";
export const EXTENTION_NAME = "Tombi";
export const SUPPORT_TOML_LANGUAGES = ["toml", "cargoLock"];
export const SUPPORT_TOMBI_CONFIG_FILENAMES = ["tombi.toml", "pyproject.toml"];
export const SUPPORT_JSON_LANGUAGES = ["json"];

export class Extension {
  private statusBarItem: vscode.StatusBarItem;

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
      EXTENTION_ID,
    ) as Settings;

    const tombiBin = await bootstrap(context, settings);

    const server = new Server(tombiBin);
    const client = new node.LanguageClient(
      EXTENTION_ID,
      `${EXTENTION_NAME} Language Server`,
      serverOptions(server.tombiBin.path, settings),
      clientOptions(),
      // biome-ignore lint/complexity/useLiteralKeys: <explanation>
      process.env["__TOMBI_LANGUAGE_SERVER_DEBUG"] !== undefined,
    );

    await client.start;

    const extenstion = new Extension(context, client, server);

    // NOTE: When VSCode starts, if a TOML document is open in a tab and the focus is not on it,
    //       the Language Server will not start.
    //       Therefore, send the notification to the Language Server for all open TOML documents.
    for (const document of vscode.workspace.textDocuments) {
      await extenstion.onDidOpenTextDocument(document);
    }

    // Update status bar for initial state
    extenstion.updateStatusBarItem();

    log.info("extension activated");

    return extenstion;
  }

  async deactivate(): Promise<void> {
    this.statusBarItem.dispose();
    await this.client.stop();
    log.info("extension deactivated");
  }

  private registerCommands(): void {
    this.context.subscriptions.push(
      vscode.commands.registerCommand(
        `${EXTENTION_ID}.showLanguageServerVersion`,
        async () => command.showLanguageServerVersion(this.server),
      ),
    );
    this.context.subscriptions.push(
      vscode.commands.registerCommand(
        `${EXTENTION_ID}.restartLanguageServer`,
        async () => command.restartLanguageServer(this.client),
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
      }),
    );
  }

  private registerExtensionSchemas(): void {
    registerExtensionSchemas(this.client);
  }

  private async updateStatusBarItem(): Promise<void> {
    const editor = vscode.window.activeTextEditor;
    if (editor && SUPPORT_TOML_LANGUAGES.includes(editor.document.languageId)) {
      try {
        const { tomlVersion, source } = await this.client.sendRequest(
          getTomlVersion,
          {
            uri: editor.document.uri.toString(),
          },
        );
        this.statusBarItem.text = `TOML: ${tomlVersion} (${source})`;
        this.statusBarItem.color = undefined;
        this.statusBarItem.backgroundColor = undefined;
        this.statusBarItem.command = `${EXTENTION_ID}.showLanguageServerVersion`;
        this.statusBarItem.show();
      } catch (error) {
        this.statusBarItem.text = "TOML: <unknown>";
        this.statusBarItem.tooltip = `${error}`;
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
