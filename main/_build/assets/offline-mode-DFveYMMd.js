import{M as s}from"./index-6OQr06v0.js";import{b as e,k as f}from"./web-DCz9_uPG.js";function c(o){const n={a:"a",code:"code",h1:"h1",p:"p",...s(),...o.components},{CodeBlock:r,Note:t,PageHeading:d}=n;return r||i("CodeBlock"),t||i("Note"),d||i("PageHeading"),[e(d,{title:"Offline Mode",description:"Use Tombi without internet connection.",og_url:"https://tombi-toml.github.io/tombi/main/docs/cli/offline-mode"}),`
`,e(n.h1,{id:"offline-mode",className:"group relative heading-with-anchor",children:"Offline Mode"}),`
`,e(n.p,{get children(){return["Tombi supports ",e(n.code,{children:"Offline Mode"}),"; you can run Tombi without fetching remote schemas."]}}),`
`,e(r,{code:"tombi format --offline",language:"bash"}),`
`,e(t,{get children(){return e(n.p,{children:`Offline mode is specified by command line arguments, not by configuration files.
It is not a team rule issue, but an issue of your working environment.
If you want to use different schema catalogs, write it in the configuration file.`})}}),`
`,e(t,{get children(){return e(n.p,{get children(){return[`If you are using VSCode or Zed, you can set offline mode in their respective settings files. See the
`,e(n.a,{href:"/tombi/main/docs/editors/vscode-extension#offline-mode",children:"VSCode Extension"}),` and the
`,e(n.a,{href:"/tombi/main/docs/editors/zed-extension#offline-mode",children:"Zed Extension"}),`
for details.`]}})}})]}function l(o={}){const{wrapper:n}={...s(),...o.components};return n?e(n,f(o,{get children(){return e(c,o)}})):c(o)}function i(o,n){throw new Error("Expected component `"+o+"` to be defined: you likely forgot to import, pass, or provide it.")}export{l as default};
