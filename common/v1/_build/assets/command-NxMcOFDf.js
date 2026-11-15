import{M as a}from"./index-6OQr06v0.js";import{b as e,k as s}from"./web-DCz9_uPG.js";function o(i){const n={code:"code",h1:"h1",h2:"h2",h3:"h3",h4:"h4",hr:"hr",li:"li",p:"p",strong:"strong",ul:"ul",...a(),...i.components},{CodeBlock:t,PageHeading:r}=n;return t||c("CodeBlock"),r||c("PageHeading"),[e(r,{title:"Command",description:"Custom LSP commands for schema and configuration management.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/language-server/command"}),`
`,e(n.h1,{id:"command",className:"group relative heading-with-anchor",children:"Command"}),`
`,e(n.p,{children:"Tombi Language Server provides custom LSP commands for advanced features like schema management and configuration updates."}),`
`,e(n.h2,{id:"available-commands",className:"group relative heading-with-anchor",children:"Available Commands"}),`
`,e(n.h3,{id:"tombi-get-status",className:"group relative heading-with-anchor",children:"tombi/getStatus"}),`
`,e(n.p,{children:"Get the current status of a TOML document, including its TOML version, configuration path, and whether it's ignored by file patterns."}),`
`,e(n.h4,{id:"request",className:"group relative heading-with-anchor",children:"Request"}),`
`,e(t,{code:`interface GetStatusParams {
  textDocument: TextDocumentIdentifier;
}`,language:"typescript"}),`
`,e(n.h4,{id:"response",className:"group relative heading-with-anchor",children:"Response"}),`
`,e(t,{code:`interface GetStatusResponse {
  tomlVersion: string;
  source: "comment" | "schema" | "config" | "default";
  configPath?: string;
  ignore?:
    | "include-file-pattern-not-matched"
    | "exclude-file-pattern-matched"
    | "ignore-file-matched";
  schema?: {
    title?: string;
    description?: string;
    uri: string;
  }
}`,language:"typescript"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"tomlVersion"}),': The TOML version used for the document (e.g., "v1.0.0", "v1.1.0")']}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"source"}),": Where the TOML version comes from:",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"comment"}),": From comment directive (",e(n.code,{children:'#:tombi toml-version = "v1.0.0"'}),")"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"schema"}),": From schema directive (",e(n.code,{children:'#:schema "https://example.com/schema.json"'}),")"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"config"}),": From configuration file"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"default"}),": Default value"]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"configPath"}),": Path to the configuration file used for this document"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"ignore"}),": Reason why the document is ignored (if applicable)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"schema"}),": Schema information (if applicable)",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"title"}),": Title of the schema (in most cases, this matches the ",e(n.code,{children:"name"})," field in the SchemaStore catalog)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"description"}),": Description of the schema (in most cases, this matches the ",e(n.code,{children:"description"})," field in the SchemaStore catalog)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"uri"}),": URI of the schema"]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.h4,{id:"example",className:"group relative heading-with-anchor",children:"Example"}),`
`,e(t,{code:'// Get the status of the current document\nconst status = await client.sendRequest("tombi/getStatus", {\n  textDocument: { uri: "file:///path/to/Cargo.toml" }\n});\n\nconsole.log(`TOML Version: ${status.tomlVersion}`);\nconsole.log(`Source: ${status.source}`);\nif (status.configPath) {\n  console.log(`Config Path: ${status.configPath}`);\n}\nif (status.ignore) {\n  console.log(`Ignored: ${status.ignore}`);\n}',language:"typescript"}),`
`,e(n.hr,{}),`
`,e(n.h3,{id:"tombi-get-toml-version",className:"group relative heading-with-anchor",children:"tombi/getTomlVersion"}),`
`,e(n.p,{children:"Get the TOML version of a document and its source."}),`
`,e(n.h4,{id:"request",className:"group relative heading-with-anchor",children:"Request"}),`
`,e(t,{code:`interface GetTomlVersionParams {
  textDocument: TextDocumentIdentifier;
}`,language:"typescript"}),`
`,e(n.h4,{id:"response",className:"group relative heading-with-anchor",children:"Response"}),`
`,e(t,{code:`interface GetTomlVersionResponse {
  tomlVersion: string;
  source: "comment" | "schema" | "config" | "default";
}`,language:"typescript"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"tomlVersion"}),": The TOML version used for the document"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"source"}),": Where the TOML version comes from (same as ",e(n.code,{children:"tombi/getStatus"}),")"]}}),`
`]}}),`
`,e(n.h4,{id:"example",className:"group relative heading-with-anchor",children:"Example"}),`
`,e(t,{code:'// Get the TOML version of the current document\nconst response = await client.sendRequest("tombi/getTomlVersion", {\n  textDocument: { uri: "file:///path/to/pyproject.toml" }\n});\n\nconsole.log(`TOML Version: ${response.tomlVersion}`);\nconsole.log(`Source: ${response.source}`);',language:"typescript"}),`
`,e(n.hr,{}),`
`,e(n.h3,{id:"tombi-list-schemas",className:"group relative heading-with-anchor",children:"tombi/listSchemas"}),`
`,e(n.p,{children:"List all available schemas known to the LSP server."}),`
`,e(n.h4,{id:"request",className:"group relative heading-with-anchor",children:"Request"}),`
`,e(t,{code:"interface ListSchemasParams {}",language:"typescript"}),`
`,e(n.h4,{id:"response",className:"group relative heading-with-anchor",children:"Response"}),`
`,e(t,{code:`interface ListSchemasResponse {
  schemas: {
    title?: string;
    description?: string;
    tomlVersion?: string;
    uri: string;
    catalogUri?: string;
  }[];
}`,language:"typescript"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"title"}),": Title of the schema (in most cases, this matches the ",e(n.code,{children:"name"})," field in the SchemaStore catalog)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"description"}),": Description of the schema (in most cases, this matches the ",e(n.code,{children:"description"})," field in the SchemaStore catalog)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"tomlVersion"}),": TOML version required by the schema"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"uri"}),": URI of the schema"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"catalogUri"}),": URI in the schema catalog (e.g., Schema Store)"]}}),`
`]}}),`
`,e(n.h4,{id:"example",className:"group relative heading-with-anchor",children:"Example"}),`
`,e(t,{code:`// List all available schemas
const response = await client.sendRequest("tombi/listSchemas", {});

console.log(\`Found \${response.schemas.length} schemas\`);
response.schemas.forEach(schema => {
  console.log(\`- \${schema.title || 'Untitled'}: \${schema.uri}\`);
  if (schema.tomlVersion) {
    console.log(\`  TOML Version: \${schema.tomlVersion}\`);
  }
});`,language:"typescript"}),`
`,e(n.hr,{}),`
`,e(n.h3,{id:"tombi-update-schema",className:"group relative heading-with-anchor",children:"tombi/updateSchema"}),`
`,e(n.p,{children:"Update a schema from its remote source. This is useful when you want to refresh the schema without restarting the language server."}),`
`,e(n.h4,{id:"request",className:"group relative heading-with-anchor",children:"Request"}),`
`,e(t,{code:`interface UpdateSchemaParams {
  textDocument: TextDocumentIdentifier;
}`,language:"typescript"}),`
`,e(n.p,{get children(){return["The ",e(n.code,{children:"textDocument"})," should point to a schema file URI."]}}),`
`,e(n.h4,{id:"response",className:"group relative heading-with-anchor",children:"Response"}),`
`,e(t,{code:"type UpdateSchemaResponse = boolean;",language:"typescript"}),`
`,e(n.p,{get children(){return["Returns ",e(n.code,{children:"true"})," if the schema was updated successfully, ",e(n.code,{children:"false"})," otherwise."]}}),`
`,e(n.h4,{id:"example",className:"group relative heading-with-anchor",children:"Example"}),`
`,e(t,{code:`// Update a schema from its remote source
const updated = await client.sendRequest("tombi/updateSchema", {
  textDocument: { uri: "https://www.schemastore.org/cargo.json" }
});

if (updated) {
  console.log("Schema updated successfully");
} else {
  console.log("Schema was not updated (already up-to-date or error occurred)");
}`,language:"typescript"}),`
`,e(n.hr,{}),`
`,e(n.h3,{id:"tombi-update-config",className:"group relative heading-with-anchor",children:"tombi/updateConfig"}),`
`,e(n.p,{children:"Update the configuration file. This command reloads the configuration from disk and applies it to the language server."}),`
`,e(n.h4,{id:"request",className:"group relative heading-with-anchor",children:"Request"}),`
`,e(t,{code:"type UpdateConfigParams = TextDocumentIdentifier;",language:"typescript"}),`
`,e(n.p,{get children(){return["The identifier should point to a Tombi configuration file (e.g., ",e(n.code,{children:"tombi.toml"}),")."]}}),`
`,e(n.h4,{id:"response",className:"group relative heading-with-anchor",children:"Response"}),`
`,e(t,{code:"type UpdateConfigResponse = boolean;",language:"typescript"}),`
`,e(n.p,{get children(){return["Returns ",e(n.code,{children:"true"})," if the configuration was updated successfully, ",e(n.code,{children:"false"})," otherwise."]}}),`
`,e(n.h4,{id:"example",className:"group relative heading-with-anchor",children:"Example"}),`
`,e(t,{code:`// Update the configuration file
const updated = await client.sendRequest("tombi/updateConfig", {
  uri: "file:///path/to/project/tombi.toml"
});

if (updated) {
  console.log("Configuration updated successfully");
} else {
  console.log("Configuration was not updated");
}`,language:"typescript"}),`
`,e(n.hr,{}),`
`,e(n.h3,{id:"tombi-associate-schema",className:"group relative heading-with-anchor",children:"tombi/associateSchema"}),`
`,e(n.p,{children:"Associate a schema with file match patterns. This is primarily used by editor extensions to dynamically register schemas."}),`
`,e(n.h4,{id:"request",className:"group relative heading-with-anchor",children:"Request"}),`
`,e(t,{code:`interface AssociateSchemaParams {
  title?: string;
  description?: string;
  uri: string;
  fileMatch: string[];
  tomlVersion?: string;
  force?: boolean;
}`,language:"typescript"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"title"}),": Title of the schema (in most cases, this matches the ",e(n.code,{children:"name"})," field in the SchemaStore catalog)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"description"}),": Description of the schema (in most cases, this matches the ",e(n.code,{children:"description"})," field in the SchemaStore catalog)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"uri"}),": URI of the schema to associate"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"fileMatch"}),": Array of glob patterns for files that should use this schema"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"tomlVersion"}),": TOML version required by the schema"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"force"}),": Determines where in the schema list the associated schema is inserted. Default is ",e(n.code,{children:"false"}),".",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["If ",e(n.code,{children:"true"}),", the schema is inserted at the ",e(n.strong,{children:"beginning"})," (highest priority, takes precedence over catalog schemas)."]}}),`
`,e(n.li,{get children(){return["If ",e(n.code,{children:"false"}),", the schema is inserted at the ",e(n.strong,{children:"end"})," (lowest priority)."]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.h4,{id:"response",className:"group relative heading-with-anchor",children:"Response"}),`
`,e(n.p,{children:"This command has no return value (notification-style)."}),`
`,e(n.h4,{id:"example",className:"group relative heading-with-anchor",children:"Example"}),`
`,e(t,{code:`// Associate a schema with file patterns
await client.sendRequest("tombi/associateSchema", {
  title: "Custom Application Config",
  description: "Schema for application configuration files",
  uri: "https://example.com/schemas/app-config.json",
  fileMatch: ["**/config.toml", "**/app.toml"],
  tomlVersion: "v1.0.0",
  force: true // This schema takes precedence over catalog schemas
});

console.log("Schema associated successfully");`,language:"typescript"}),`
`,e(n.p,{get children(){return["VSCode extensions can also contribute schema associations via ",e(n.code,{children:"package.json"}),":"]}}),`
`,e(t,{code:`{
  "contributes": {
    "tomlValidation": [
      {
        "regexMatch": "^.*foo.toml$",
        "url": "https://www.schemastore.org/foo.json"
      }
    ]
  }
}`,language:"json"}),`
`,e(n.hr,{}),`
`,e(n.h3,{id:"tombi-refresh-cache",className:"group relative heading-with-anchor",children:"tombi/refreshCache"}),`
`,e(n.p,{children:"Refresh the schema cache. This command clears and rebuilds the internal schema cache."}),`
`,e(n.h4,{id:"request",className:"group relative heading-with-anchor",children:"Request"}),`
`,e(t,{code:"interface RefreshCacheParams {}",language:"typescript"}),`
`,e(n.h4,{id:"response",className:"group relative heading-with-anchor",children:"Response"}),`
`,e(t,{code:"type RefreshCacheResponse = boolean;",language:"typescript"}),`
`,e(n.p,{get children(){return["Returns ",e(n.code,{children:"true"})," if the cache was refreshed successfully, ",e(n.code,{children:"false"})," if there was no cache to refresh."]}}),`
`,e(n.h4,{id:"example",className:"group relative heading-with-anchor",children:"Example"}),`
`,e(t,{code:`// Refresh the schema cache
const refreshed = await client.sendRequest("tombi/refreshCache", {});

if (refreshed) {
  console.log("Cache refreshed successfully");
} else {
  console.log("No cache to refresh");
}`,language:"typescript"})]}function d(i={}){const{wrapper:n}={...a(),...i.components};return n?e(n,s(i,{get children(){return e(o,i)}})):o(i)}function c(i,n){throw new Error("Expected component `"+i+"` to be defined: you likely forgot to import, pass, or provide it.")}export{d as default};
