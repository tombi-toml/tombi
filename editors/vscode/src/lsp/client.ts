import {
  RequestType,
  type TextDocumentIdentifier,
} from "vscode-languageclient";

export type TomlVersionSource = "comment" | "schema" | "config" | "default";
export type IgnoreReason =
  | "include-file-pattern-not-matched"
  | "exclude-file-pattern-matched";

export type GetTomlVersionParams = TextDocumentIdentifier;
export const getTomlVersion = new RequestType<
  GetTomlVersionParams,
  { tomlVersion: string; source: TomlVersionSource },
  void
>("tombi/getTomlVersion");

export type UpdateConfigParams = TextDocumentIdentifier;
export const updateConfig = new RequestType<UpdateConfigParams, boolean, void>(
  "tombi/updateConfig",
);

export type UpdateSchemaParams = TextDocumentIdentifier;
export const updateSchema = new RequestType<UpdateSchemaParams, boolean, void>(
  "tombi/updateSchema",
);

export type AssociateSchemaParams = {
  title?: string;
  description?: string;
  uri: string;
  fileMatch: string[];
  tomlVersion?: string;
  /**
   * If true, the schema will be inserted at the beginning of the schema list
   * to force it to take precedence over catalog schemas. Default is false.
   */
  force?: boolean;
};
export const associateSchema = new RequestType<
  AssociateSchemaParams,
  void,
  void
>("tombi/associateSchema");

export type SchemaInfo = {
  title?: string;
  description?: string;
  tomlVersion?: string;
  uri: string;
  catalogUri?: string;
};
export const listSchemas = new RequestType<
  void,
  { schemas: SchemaInfo[] },
  void
>("tombi/listSchemas");

export type GetStatusParams = TextDocumentIdentifier;
export const getStatus = new RequestType<
  GetStatusParams,
  {
    tomlVersion: string;
    source: TomlVersionSource;
    configPath?: string;
    ignore?: IgnoreReason;
  },
  void
>("tombi/getStatus");
