import{M as m}from"./index-6OQr06v0.js";import{b as t,k as c}from"./web-DCz9_uPG.js";function i(o){const e={h1:"h1",p:"p",...m(),...o.components},{CodeBlock:n,PageHeading:r}=e;return n||a("CodeBlock"),r||a("PageHeading"),[t(r,{title:"Formatting",description:"Format TOML files via Language Server Protocol.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/language-server/formatting"}),`
`,t(e.h1,{id:"formatting",className:"group relative heading-with-anchor",children:"Formatting"}),`
`,t(e.p,{children:"When using Tombi as a formatter in VSCode, please set the following configuration."}),`
`,t(n,{code:`{
  "editor.formatOnSave": true,
  "[toml]": {
    "editor.defaultFormatter": "tombi-toml.tombi"
  }
}`,language:"json"})]}function l(o={}){const{wrapper:e}={...m(),...o.components};return e?t(e,c(o,{get children(){return t(i,o)}})):i(o)}function a(o,e){throw new Error("Expected component `"+o+"` to be defined: you likely forgot to import, pass, or provide it.")}export{l as default};
