import{M as s}from"./index-6OQr06v0.js";import{b as e,k as d}from"./web-DCz9_uPG.js";function a(r){const n={code:"code",h1:"h1",h2:"h2",p:"p",...s(),...r.components},{CodeBlock:t,PageHeading:o}=n;return t||i("CodeBlock"),o||i("PageHeading"),[e(o,{title:"Language Server",description:"Use the tombi-wasm-lsp package and its Web Worker adapter.",og_url:"https://tombi-toml.github.io/tombi/main/docs/wasm/tombi-wasm-lsp"}),`
`,e(n.h1,{id:"tombi-wasm-lsp",className:"group relative heading-with-anchor",get children(){return e(n.code,{children:"tombi-wasm-lsp"})}}),`
`,e(n.p,{get children(){return[e(n.code,{children:"tombi-wasm-lsp"}),` provides Tombi's Language Server for browsers and other JavaScript
environments. It also includes a Web Worker adapter at
`,e(n.code,{children:"tombi-wasm-lsp/worker"}),"."]}}),`
`,e(n.h2,{id:"install",className:"group relative heading-with-anchor",children:"Install"}),`
`,e(t,{code:"pnpm add tombi-wasm-lsp",language:"sh"}),`
`,e(n.h2,{id:"run-the-language-server",className:"group relative heading-with-anchor",children:"Run the Language Server"}),`
`,e(n.p,{get children(){return["The package exposes ",e(n.code,{children:"serve"}),` for hosts that provide browser-compatible streams.
`,e(n.code,{children:"ServerConfig"})," receives an async iterator of ",e(n.code,{children:"Uint8Array"}),` input chunks and a
`,e(n.code,{children:"WritableStream"})," for output:"]}}),`
`,e(t,{code:`import init, { ServerConfig, serve } from "tombi-wasm-lsp";

await init();

const input = new ReadableStream<Uint8Array>({
  start(controller) {
    // Enqueue LSP-framed messages here.
  },
});
const output = new WritableStream<Uint8Array>({
  write(chunk) {
    // Read LSP-framed responses here.
    console.log(chunk);
  },
});

await serve(new ServerConfig(input.values(), output));`,language:"ts"}),`
`,e(n.p,{get children(){return["Messages use standard LSP JSON-RPC framing (",e(n.code,{children:"Content-Length"}),` followed by a JSON
message). Browser hosts should represent the virtual workspace with
`,e(n.code,{children:"textDocument/didOpen"}),", synchronize edits with ",e(n.code,{children:"textDocument/didChange"}),`, and send
`,e(n.code,{children:"textDocument/didSave"})," when appropriate."]}}),`
`,e(n.p,{get children(){return[`Before starting a server, preload files that are not opened in the editor with
`,e(n.code,{children:"set_workspace_entries"}),":"]}}),`
`,e(t,{code:`import { set_workspace_entries } from "tombi-wasm-lsp";

set_workspace_entries([
  {
    uri: "file:///workspace/tombi.toml",
    text: "[format]\\nnewline = \\"lf\\"\\n",
  },
]);`,language:"ts"}),`
`,e(n.p,{get children(){return["Use ",e(n.code,{children:'kind: "directory"'}),` for directory entries; file entries default to
`,e(n.code,{children:'kind: "file"'}),"."]}}),`
`,e(n.h2,{id:"run-in-a-web-worker",className:"group relative heading-with-anchor",children:"Run in a Web Worker"}),`
`,e(n.p,{children:`For a browser editor, run the Language Server in a Web Worker so parsing and
diagnostics do not block the UI thread:`}),`
`,e(t,{code:`const worker = new Worker(
  new URL("tombi-wasm-lsp/worker", import.meta.url),
  { type: "module" },
);

worker.addEventListener("message", (event) => {
  if (event.data.type === "ready") {
    // Send LSP JSON-RPC messages with worker.postMessage(...).
  }
});`,language:"ts"}),`
`,e(n.p,{get children(){return["The worker forwards messages through ",e(n.code,{children:"postMessage"})," and emits ",e(n.code,{children:'{ type: "ready" }'}),`
after the WASM module and Language Server have started. It accepts standard LSP
JSON-RPC messages and mirrors opened `,e(n.code,{children:"file:"})," documents into the in-memory workspace."]}})]}function h(r={}){const{wrapper:n}={...s(),...r.components};return n?e(n,d(r,{get children(){return e(a,r)}})):a(r)}function i(r,n){throw new Error("Expected component `"+r+"` to be defined: you likely forgot to import, pass, or provide it.")}export{h as default};
