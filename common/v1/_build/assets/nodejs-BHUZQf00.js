import{M as c}from"./index-6OQr06v0.js";import{b as e,k as s}from"./web-DCz9_uPG.js";function r(o){const n={a:"a",code:"code",h1:"h1",h2:"h2",p:"p",...c(),...o.components},{CodeBlock:t,PageHeading:i}=n;return t||a("CodeBlock"),i||a("PageHeading"),[e(i,{title:"Node.js",description:"Use the tombi-lib package from npm.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/library/nodejs"}),`
`,e(n.h1,{id:"node-js",className:"group relative heading-with-anchor",children:"Node.js"}),`
`,e(n.p,{get children(){return[e(n.code,{children:"tombi-lib"}),` is a native Node.js addon for server-side JavaScript such as CLI tools,
build scripts, and servers. It runs without WASM initialization and can read
`,e(n.code,{children:"tombi.toml"})," from the file system."]}}),`
`,e(n.p,{get children(){return["To run Tombi in a browser, use ",e(n.a,{href:"/tombi/common/v1/docs/wasm/tombi-wasm-lib",get children(){return e(n.code,{children:"tombi-wasm-lib"})}}),`
instead.`]}}),`
`,e(n.h2,{id:"install",className:"group relative heading-with-anchor",children:"Install"}),`
`,e(t,{code:"npm install tombi-lib",language:"sh"}),`
`,e(n.h2,{id:"usage",className:"group relative heading-with-anchor",children:"Usage"}),`
`,e(t,{code:`import { format, lint } from "tombi-lib";

const { formatted } = await format("key=1", "example.toml");
console.log(formatted); // key = 1

const { diagnostics } = await lint("key =", "example.toml");
for (const diagnostic of diagnostics) {
  console.log(diagnostic.level, diagnostic.message);
}`,language:"ts"}),`
`,e(n.p,{get children(){return[e(n.code,{children:"formatSync"})," / ",e(n.code,{children:"lintSync"})," are also available when a synchronous call is needed:"]}}),`
`,e(t,{code:`import { formatSync } from "tombi-lib";

const { formatted } = formatSync("key=1", "example.toml");`,language:"ts"}),`
`,e(n.h2,{id:"configuration",className:"group relative heading-with-anchor",children:"Configuration"}),`
`,e(n.p,{get children(){return["By default, ",e(n.code,{children:"tombi.toml"}),` is searched from the current directory.
Pass `,e(n.code,{children:"config"})," to use an in-memory configuration instead:"]}}),`
`,e(t,{code:'await format(source, "Cargo.toml", { config: "[format.rules]\\nindent-width = 4\\n" });',language:"ts"})]}function m(o={}){const{wrapper:n}={...c(),...o.components};return n?e(n,s(o,{get children(){return e(r,o)}})):r(o)}function a(o,n){throw new Error("Expected component `"+o+"` to be defined: you likely forgot to import, pass, or provide it.")}export{m as default};
