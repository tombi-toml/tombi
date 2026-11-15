import{M as s}from"./index-6OQr06v0.js";import{b as e,k as f}from"./web-DCz9_uPG.js";function d(n){const o={a:"a",code:"code",h1:"h1",p:"p",...s(),...n.components},{CodeBlock:r,Note:t,PageHeading:c}=o;return r||i("CodeBlock"),t||i("Note"),c||i("PageHeading"),[e(c,{title:"Offline Mode",description:"Use Tombi without internet connection.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/cli/offline-mode"}),`
`,e(o.h1,{id:"offline-mode",className:"group relative heading-with-anchor",children:"Offline Mode"}),`
`,e(o.p,{get children(){return["Tombi supports ",e(o.code,{children:"Offline Mode"}),"; you can run Tombi without fetching remote schemas."]}}),`
`,e(r,{code:"tombi format --offline",language:"bash"}),`
`,e(t,{get children(){return e(o.p,{children:`Offline mode is specified by command line arguments, not by configuration files.
It is not a team rule issue, but an issue of your working environment.
If you want to use different schema catalogs, write it in the configuration file.`})}}),`
`,e(t,{get children(){return e(o.p,{get children(){return[`If you are using VSCode or Zed, you can set offline mode in their respective settings files. See the
`,e(o.a,{href:"/tombi/common/v1/docs/editors/vscode-extension#offline-mode",children:"VSCode Extension"}),` and the
`,e(o.a,{href:"/tombi/common/v1/docs/editors/zed-extension#offline-mode",children:"Zed Extension"}),`
for details.`]}})}})]}function l(n={}){const{wrapper:o}={...s(),...n.components};return o?e(o,f(n,{get children(){return e(d,n)}})):d(n)}function i(n,o){throw new Error("Expected component `"+n+"` to be defined: you likely forgot to import, pass, or provide it.")}export{l as default};
