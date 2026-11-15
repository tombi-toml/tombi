import{M as a}from"./index-6OQr06v0.js";import{b as e,k as l}from"./web-DCz9_uPG.js";function o(t){const n={a:"a",code:"code",h1:"h1",h2:"h2",h3:"h3",p:"p",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",...a(),...t.components},{CodeBlock:i,PageHeading:r}=n;return i||d("CodeBlock"),r||d("PageHeading"),[e(r,{title:"Zed Extension",description:"Tombi extension for Zed editor.",og_url:"https://tombi-toml.github.io/tombi/main/docs/editors/zed-extension"}),`
`,e(n.h1,{id:"zed-extension",className:"group relative heading-with-anchor",children:"Zed Extension"}),`
`,e(n.p,{get children(){return[`The Tombi extension currently published in Zed's extension registry focuses on Language Server Protocol (LSP) features.
Syntax highlighting and other syntax-related editor features are provided by the `,e(n.a,{href:"https://github.com/zed-extensions/toml",children:"TOML extension"}),", which should be installed alongside the Tombi extension."]}}),`
`,e(n.h2,{id:"language-server-binary-priority",className:"group relative heading-with-anchor",children:"Language Server Binary Priority"}),`
`,e(n.p,{children:"The tombi Zed extension selects the Language Server binary in the following order of priority:"}),`
`,e(n.table,{get children(){return[e(n.thead,{get children(){return e(n.tr,{get children(){return[e(n.th,{children:"Order"}),e(n.th,{children:"Source Type"}),e(n.th,{children:"Description"})]}})}}),e(n.tbody,{get children(){return[e(n.tr,{get children(){return[e(n.td,{children:"1"}),e(n.td,{children:"editor settings"}),e(n.td,{get children(){return["Path explicitly specified by ",e(n.code,{children:"lsp.tombi.binary.path"})," in Zed settings"]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"2"}),e(n.td,{children:"venv"}),e(n.td,{get children(){return[e(n.code,{children:"tombi"})," binary found in a Python virtual environment (",e(n.code,{children:".venv/bin/tombi"})," or ",e(n.code,{children:".venv/Scripts/tombi.exe"}),") in the workspace"]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"3"}),e(n.td,{children:"node_modules"}),e(n.td,{get children(){return[e(n.code,{children:"tombi"})," binary found in the ",e(n.code,{children:"node_modules/.bin"})," directory in the workspace"]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"4"}),e(n.td,{children:"local"}),e(n.td,{get children(){return[e(n.code,{children:"tombi"})," binary found in the system PATH (via ",e(n.code,{children:"which tombi"}),")"]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"5"}),e(n.td,{children:"cached"}),e(n.td,{get children(){return[e(n.code,{children:"tombi"})," binary cached in the current Zed process from a previous extension-managed resolution"]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"6"}),e(n.td,{children:"downloaded"}),e(n.td,{get children(){return["Download the latest ",e(n.code,{children:"tombi"})," binary from GitHub"]}})]}}),e(n.tr,{get children(){return[e(n.td,{children:"7"}),e(n.td,{children:"fallback"}),e(n.td,{get children(){return["If the download fails, use the newest already-downloaded extension-managed ",e(n.code,{children:"tombi"})," binary"]}})]}})]}})]}}),`
`,e(n.p,{children:"When the extension needs to manage the binary itself for the first time in a given Zed process, step 5 is skipped because no process-local cache exists yet. In that case, the extension attempts step 6 first and only falls back to step 7 if the download fails."}),`
`,e(n.h2,{id:"configuration",className:"group relative heading-with-anchor",children:"Configuration"}),`
`,e(n.p,{children:"You can configure the extension through Zed's LSP settings. For example, to specify custom arguments and environment variables:"}),`
`,e(i,{code:`{
  "lsp": {
    "tombi": {
      "binary": {
        "arguments": ["lsp", "-v"],
        "env": { "NO_COLOR": "true" }
      }
    }
  }
}`,language:"json"}),`
`,e(n.h3,{id:"format-settings",className:"group relative heading-with-anchor",children:"Format settings"}),`
`,e(n.p,{children:"If you cannot format your TOML files, please try to set the following settings."}),`
`,e(i,{code:`{
  "lsp": {
    "tombi": {
      "binary": {
        "arguments": ["lsp", "-vv"],
        "env": { "NO_COLOR": "true" }
      }
    }
  }
}`,language:"json"}),`
`,e(n.h3,{id:"offline-mode",className:"group relative heading-with-anchor",children:"Offline Mode"}),`
`,e(n.p,{children:"You can set offline mode in the Zed settings file."}),`
`,e(i,{code:`{
  "lsp": {
    "tombi": {
      "binary": {
        "arguments": ["lsp", "--offline"]
      }
    }
  }
}`,language:"json"}),`
`,e(n.h2,{id:"troubleshooting",className:"group relative heading-with-anchor",children:"Troubleshooting"}),`
`,e(n.p,{get children(){return["Zed v1.1.6 and later include a workaround for background type definition requests that could open unwanted ",e(n.code,{children:"lsp edit"}),` tabs.
If you still encounter this behavior, you can temporarily disable Tombi's Go to Type Definition feature in `,e(n.code,{children:"tombi.toml"}),":"]}}),`
`,e(i,{code:`[lsp]
goto-type-definition.enabled = false`,language:"toml"}),`
`,e(n.p,{get children(){return["For details and alternative workarounds, see ",e(n.a,{href:"https://github.com/tombi-toml/tombi/issues/1556",children:"#1556"}),"."]}})]}function s(t={}){const{wrapper:n}={...a(),...t.components};return n?e(n,l(t,{get children(){return e(o,t)}})):o(t)}function d(t,n){throw new Error("Expected component `"+t+"` to be defined: you likely forgot to import, pass, or provide it.")}export{s as default};
