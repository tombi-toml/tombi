import{b as e,k as s,i as o,t as d}from"./web-DCz9_uPG.js";import{M as l}from"./index-6OQr06v0.js";var h=d("<img src=/tombi/main/show_language_server_version.png class=w-full>"),g=d("<img src=/tombi/main/select_schema.png class=w-full>");function a(t){const n={a:"a",code:"code",h1:"h1",h2:"h2",li:"li",p:"p",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",ul:"ul",...l(),...t.components},{CodeBlock:i,PageHeading:r}=n;return i||c("CodeBlock"),r||c("PageHeading"),[e(r,{title:"VSCode Extension",description:"Tombi extension for Visual Studio Code.",og_url:"https://tombi-toml.github.io/tombi/main/docs/editors/vscode-extension"}),`
`,e(n.h1,{id:"vscode-extension",className:"group relative heading-with-anchor",children:"VSCode Extension"}),`
`,e(n.h2,{id:"language-server-binary-priority",className:"group relative heading-with-anchor",children:"Language Server Binary Priority"}),`
`,e(n.p,{children:"The tombi VSCode extension selects the Language Server binary in the following order of priority:"}),`
`,e(n.table,{get children(){return[e(n.thead,{get children(){return e(n.tr,{get children(){return[e(n.th,{children:"Order"}),e(n.th,{children:"Source Type"}),e(n.th,{children:"Description"})]}})}}),e(n.tbody,{get children(){return[e(n.tr,{get children(){return[e(n.td,{children:"1"}),e(n.td,{children:"editor settings"}),e(n.td,{get children(){return["Path explicitly specified by ",e(n.code,{children:"tombi.path"})," in ",e(n.code,{children:".vscode/settings.json"})]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"2"}),e(n.td,{children:"venv"}),e(n.td,{get children(){return[e(n.code,{children:"tombi"})," binary found in a Python virtual environment (venv) in the workspace"]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"3"}),e(n.td,{children:"node_modules"}),e(n.td,{get children(){return[e(n.code,{children:"tombi"})," binary found in the ",e(n.code,{children:"node_modules/.bin"})," directory in the workspace"]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"4"}),e(n.td,{children:"local"}),e(n.td,{get children(){return[e(n.code,{children:"tombi"})," binary found in the system PATH"]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"5"}),e(n.td,{children:"bundled"}),e(n.td,{get children(){return[e(n.code,{children:"tombi"})," binary bundled with the extension"]}})]}})]}})]}}),`
`,e(n.p,{get children(){return["You can check which tombi binary is currently selected by running the ",e(n.code,{children:"Tombi: Show Language Server Version"})," command from the VSCode command palette."]}}),`
`,o(h),`
`,e(n.h2,{id:"select-schema",className:"group relative heading-with-anchor",children:"Select Schema"}),`
`,e(n.p,{get children(){return["You can manually select and apply a JSON schema to the current TOML file using the ",e(n.code,{children:"Tombi: Select Schema"})," command from the VSCode command palette. This command lists all available schemas and associates the selected one with the current file."]}}),`
`,e(n.p,{get children(){return["This is useful when working on projects without Tombi configuration, or when validating files whose paths don't match SchemaStore's fileMatch patterns (e.g., template files like ",e(n.code,{children:"config.toml.template"})," or example files like ",e(n.code,{children:"example-config.toml"}),"). The selected schema takes precedence over catalog schemas and automatic associations."]}}),`
`,o(g),`
`,e(n.h2,{id:"offline-mode",className:"group relative heading-with-anchor",children:"Offline Mode"}),`
`,e(n.p,{children:"You can set offline mode in the VSCode settings file."}),`
`,e(i,{code:`{
  "tombi.args": ["lsp", "--offline"]
}`,language:"json"}),`
`,e(n.h2,{id:"verbose-logging",className:"group relative heading-with-anchor",children:"Verbose Logging"}),`
`,e(n.p,{children:"You can enable verbose logging in the VSCode settings file."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Debug: ",e(n.code,{children:"-v"})]}}),`
`,e(n.li,{get children(){return["Trace: ",e(n.code,{children:"-vv"})]}}),`
`]}}),`
`,e(i,{code:`{
  "tombi.args": ["lsp", "-vv"]
}`,language:"json"}),`
`,e(n.h2,{id:"quiet-logging",className:"group relative heading-with-anchor",children:"Quiet Logging"}),`
`,e(n.p,{children:`The language server logs every request at Info level to stderr.
Some editors record all of stderr as errors, so you can lower the level.`}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Warn: ",e(n.code,{children:"-q"})]}}),`
`,e(n.li,{get children(){return["Error: ",e(n.code,{children:"-qq"})]}}),`
`]}}),`
`,e(i,{code:`{
  "tombi.args": ["lsp", "-q"]
}`,language:"json"}),`
`,e(n.h2,{id:"semantic-tokens-for-strings-and-comments",className:"group relative heading-with-anchor",children:"Semantic Tokens for Strings and Comments"}),`
`,e(n.p,{children:"Tombi sends semantic tokens for strings and comments by default. Semantic tokens take priority over TextMate grammars and other extensions' highlighting, so highlighting injected into strings by another extension (e.g. shell syntax in a string) can be hidden. You can stop Tombi from sending them in the VSCode settings file."}),`
`,e(i,{code:`{
  "tombi.semanticTokens.strings.enabled": false,
  "tombi.semanticTokens.comments.enabled": false
}`,language:"json"}),`
`,e(n.p,{get children(){return["Both settings default to ",e(n.code,{children:"true"}),". Restart the language server (",e(n.code,{children:"Tombi: Restart Language Server"}),") after changing them."]}}),`
`,e(n.p,{get children(){return["When comments are disabled, Tombi directives (",e(n.code,{children:"#:tombi ..."}),", ",e(n.code,{children:"# tombi: ..."}),") and ",e(n.code,{children:"# region"})," / ",e(n.code,{children:"# endregion"})," markers keep their keyword highlighting."]}}),`
`,e(n.p,{get children(){return["Other editors can pass the same options as ",e(n.code,{children:"initializationOptions"})," of the ",e(n.code,{children:"initialize"})," request:"]}}),`
`,e(i,{code:`{
  "semanticTokens": {
    "strings": { "enabled": true },
    "comments": { "enabled": true }
  }
}`,language:"json"}),`
`,e(n.h2,{id:"json-schema-association",className:"group relative heading-with-anchor",children:"JSON Schema Association"}),`
`,e(n.p,{get children(){return["As with ",e(n.a,{href:"https://taplo.tamasfe.dev/configuration/developing-schemas.html#visual-studio-code-extensions",children:"Taplo"}),", extensions can associate their own schemas using ",e(n.code,{children:"tomlValidation"}),"."]}}),`
`,e(n.p,{get children(){return["You can specify ",e(n.code,{children:"fileMatch"}),", which associates the schema with documents whose file matches the given pattern (same as ",e(n.a,{href:"https://code.visualstudio.com/api/references/contribution-points#contributes.jsonValidation",children:"jsonValidation"}),")."]}}),`
`,e(i,{code:`{
  "contributes": {
    "tomlValidation": [
      {
        "fileMatch": "^.*foo.toml$",
        "url": "https://www.schemastore.org/foo.json"
      }
    ]
  }
}`,language:"json"})]}function p(t={}){const{wrapper:n}={...l(),...t.components};return n?e(n,s(t,{get children(){return e(a,t)}})):a(t)}function c(t,n){throw new Error("Expected component `"+t+"` to be defined: you likely forgot to import, pass, or provide it.")}export{p as default};
