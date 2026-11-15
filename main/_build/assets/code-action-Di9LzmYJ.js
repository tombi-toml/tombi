import{M as c}from"./index-6OQr06v0.js";import{T as d}from"./Highlight-CMhf5ZLw.js";import{b as e,k as l}from"./web-DCz9_uPG.js";function r(o){const n={code:"code",h1:"h1",h2:"h2",h3:"h3",li:"li",p:"p",strong:"strong",ul:"ul",...c(),...o.components},{CodeBlock:t,PageHeading:i}=n;return t||a("CodeBlock"),i||a("PageHeading"),[e(i,{title:"Code Action",description:"Quick fixes and refactoring suggestions for TOML.",og_url:"https://tombi-toml.github.io/tombi/main/docs/language-server/code-action"}),`
`,`
`,e(n.h1,{id:"code-action",className:"group relative heading-with-anchor",children:"Code Action"}),`
`,e(n.p,{children:"Code actions provide quick refactoring operations for TOML files."}),`
`,e(n.h2,{id:"available-code-actions",className:"group relative heading-with-anchor",children:"Available Code Actions"}),`
`,e(n.h3,{id:"convert-between-dotted-keys-and-inline-tables",className:"group relative heading-with-anchor",children:"Convert between Dotted Keys and Inline Tables"}),`
`,e(n.p,{children:"Transform between Dotted Key format and Inline Table format:"}),`
`,e(t,{code:`# Dotted key
foo.bar = 1

# Can be converted to inline table
foo = { bar = 1 }`,language:"toml"}),`
`,e(d,{get children(){return e(n.p,{children:"This conversion is only available when there's a single key-value pair in the structure."})}}),`
`,e(n.h2,{id:"configuration",className:"group relative heading-with-anchor",children:"Configuration"}),`
`,e(n.p,{children:"Code actions can be enabled or disabled in your configuration:"}),`
`,e(t,{code:`[lsp]
code-action.enabled = true  # Default: true`,language:"toml"}),`
`,e(n.h2,{id:"how-to-use",className:"group relative heading-with-anchor",children:"How to Use"}),`
`,e(n.p,{children:"Code actions are triggered differently depending on your editor:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.strong,{children:"VS Code"}),": Click on the lightbulb icon that appears when your cursor is on a supported structure, or use the keyboard shortcut ",e(n.code,{children:"Ctrl+."})," (Windows/Linux) or ",e(n.code,{children:"Cmd+."})," (macOS)"]}}),`
`,e(n.li,{get children(){return[e(n.strong,{children:"Neovim"}),": Use the ",e(n.code,{children:":lua vim.lsp.buf.code_action()"})," command or bind it to a key"]}}),`
`,e(n.li,{get children(){return[e(n.strong,{children:"Other LSP-compatible editors"}),": Refer to your editor's documentation for triggering code actions"]}}),`
`]}})]}function g(o={}){const{wrapper:n}={...c(),...o.components};return n?e(n,l(o,{get children(){return e(r,o)}})):r(o)}function a(o,n){throw new Error("Expected component `"+o+"` to be defined: you likely forgot to import, pass, or provide it.")}export{g as default};
