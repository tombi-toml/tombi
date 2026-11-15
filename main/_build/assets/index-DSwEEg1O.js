import{b as t,k as l,i as c,t as s}from"./web-DCz9_uPG.js";import{M as d}from"./index-6OQr06v0.js";var m=s('<a href=/tombi/main/playground class="inline-block px-4 py-2 rounded bg-[--color-primary] text-white no-underline hover:opacity-90">Try the WASM Playground');function o(n){const e={a:"a",code:"code",h1:"h1",p:"p",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",...d(),...n.components},{CodeBlock:r,PageHeading:i}=e;return r||a("CodeBlock"),i||a("PageHeading"),[t(i,{title:"WebAssembly",description:"Run Tombi's formatter, linter, and Language Server in browsers.",og_url:"https://tombi-toml.github.io/tombi/main/docs/wasm"}),`
`,t(e.h1,{id:"web-assembly",className:"group relative heading-with-anchor",children:"WebAssembly"}),`
`,t(e.p,{children:`Tombi's WebAssembly support is published as two npm packages. They are separated so
an application only downloads the runtime it needs:`}),`
`,t(e.table,{get children(){return[t(e.thead,{get children(){return t(e.tr,{get children(){return[t(e.th,{children:"Package"}),t(e.th,{children:"Contents"})]}})}}),t(e.tbody,{get children(){return[t(e.tr,{get children(){return[t(e.td,{get children(){return t(e.a,{href:"/tombi/main/docs/wasm/tombi-wasm-lib",get children(){return t(e.code,{children:"tombi-wasm-lib"})}})}}),t(e.td,{children:"Formatter and linter"})]}}),t(e.tr,{get children(){return[t(e.td,{get children(){return t(e.a,{href:"/tombi/main/docs/wasm/tombi-wasm-lsp",get children(){return t(e.code,{children:"tombi-wasm-lsp"})}})}}),t(e.td,{children:"Language Server"})]}})]}})]}}),`
`,t(e.p,{children:"Install only the package required by your application:"}),`
`,t(r,{code:`pnpm add tombi-wasm-lib
pnpm add tombi-wasm-lsp`,language:"sh"}),`
`,t(e.p,{get children(){return[`These packages are intended for browsers. For formatting and linting in
server-side Node.js, use the native `,t(e.a,{href:"/tombi/main/docs/library/nodejs",get children(){return t(e.code,{children:"tombi-lib"})}}),` package
instead of `,t(e.code,{children:"tombi-wasm-lib"}),"."]}}),`
`,t(e.p,{get children(){return["The two packages are built from the shared ",t(e.code,{children:"tombi-wasm"})," Rust crate. The ",t(e.code,{children:"lib"}),` and
`,t(e.code,{children:"lsp"})," features produce independent WASM artifacts."]}}),`
`,c(m)]}function b(n={}){const{wrapper:e}={...d(),...n.components};return e?t(e,l(n,{get children(){return t(o,n)}})):o(n)}function a(n,e){throw new Error("Expected component `"+n+"` to be defined: you likely forgot to import, pass, or provide it.")}export{b as default};
