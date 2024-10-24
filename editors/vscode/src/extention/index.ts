import * as vscode from "vscode";
import * as node from "vscode-languageclient/node";
import { clientOptions } from "@/options/client-options";
import { serverOptions } from "@/options/server-options";
import { Server } from "@/lsp/server";
import type { Settings } from "./settings";
import * as command from "@/command";
import { bootstrap } from "@/bootstrap";
import { log } from "@/logging";
export type { Settings };

export const EXTENTION_ID = "tombi";
export const EXTENTION_NAME = "Tombi";
export const SUPPORT_LANGUAGES = ["toml", "cargoLock"];

export class Extension {
  constructor(
    private context: vscode.ExtensionContext,
    private client: node.LanguageClient,
    private server: Server,
  ) {
    vscode.languages.registerDocumentFormattingEditProvider("toml", {
      async provideDocumentFormattingEdits(
        document: vscode.TextDocument,
      ): Promise<vscode.TextEdit[]> {
        log.info("provideDocumentFormattingEdits", document.languageId);
        const response =
          (await client.sendRequest(node.DocumentFormattingRequest.type, {
            textDocument: { uri: document.uri.toString() },
            options: {
              tabSize: 4,
              insertSpaces: true,
            },
          })) || [];
        log.info("response", response);
        return response.map((edit) => {
          return new vscode.TextEdit(
            new vscode.Range(
              edit.range.start.line,
              edit.range.start.character,
              edit.range.end.line,
              edit.range.end.character,
            ),
            edit.newText,
          );
        });
      },
    });
    this.registerEvents();
    this.registerCommands();
  }

  static async activate(context: vscode.ExtensionContext): Promise<Extension> {
    const serverPath = await bootstrap(context, {});
    const server = new Server(serverPath);
    const client = new node.LanguageClient(
      EXTENTION_ID,
      EXTENTION_NAME,
      serverOptions(server.binPath),
      clientOptions(),
    );

    const extenstion = new Extension(context, client, server);
    log.info("extension started");

    return extenstion;
  }

  async deactivate(): Promise<void> {
    await this.client?.stop();
  }

  private registerCommands(): void {
    this.context.subscriptions.push(
      vscode.commands.registerCommand(
        `${EXTENTION_ID}.showServerVersion`,
        async () => {
          await command.showServerVersion(this.server);
        },
      ),
    );
  }

  private registerEvents(): void {
    vscode.workspace.onDidChangeTextDocument(
      async (event) => await this.onDidChangeTextDocument(event),
    );
    vscode.workspace.onDidSaveTextDocument(
      async (event) => await this.onDidSaveTextDocument(event),
    );
    vscode.workspace.onDidChangeConfiguration(
      async (event) => await this.onDidChangeConfiguration(event),
      null,
      this.context.subscriptions,
    );
  }

  private async onDidChangeTextDocument({
    document,
  }: vscode.TextDocumentChangeEvent): Promise<void> {
    if (!SUPPORT_LANGUAGES.includes(document.languageId)) {
      return;
    }
  }

  private async onDidChangeConfiguration(
    _: vscode.ConfigurationChangeEvent,
  ): Promise<void> {
    this.client?.sendNotification(
      node.DidChangeConfigurationNotification.type,
      {
        settings: EXTENTION_ID,
      },
    );
  }

  private async onDidSaveTextDocument(
    document: vscode.TextDocument,
  ): Promise<void> {
    log.info("onDidSaveTextDocument", document.languageId);
    if (SUPPORT_LANGUAGES.includes(document.languageId)) return;
  }
}