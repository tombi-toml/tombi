import{b as o,k as c,i as a,t as s}from"./web-DCz9_uPG.js";import{M as r}from"./index-6OQr06v0.js";var d=s('<div class="grid grid-cols-1 md:grid-cols-2 gap-4"><img src=/tombi/main/goto-type-definition.png alt="Go to Type Definition"class=w-full><img src=/tombi/main/goto-type-definition-result.png alt="Go to Type Definition"class=w-full>');function n(t){const e={a:"a",code:"code",h1:"h1",h2:"h2",li:"li",p:"p",ul:"ul",...r(),...t.components},{PageHeading:i}=e;return i||l("PageHeading"),[o(i,{title:"Go to Definition",description:"Navigate to schema definitions and references.",og_url:"https://tombi-toml.github.io/tombi/main/docs/language-server/goto-definition"}),`
`,o(e.h1,{id:"go-to-definition",className:"group relative heading-with-anchor",children:"Go to Definition"}),`
`,o(e.p,{children:"Tombi supports go to definition."}),`
`,o(e.h2,{id:"go-to-json-schema",className:"group relative heading-with-anchor",children:"Go to JSON Schema"}),`
`,o(e.p,{children:"In TOML files with an applied JSON Schema, 'Go to Type Definition' jumps to the TOML file where the JSON Schema is defined."}),`
`,a(d),`
`,o(e.h2,{id:"go-to-workspace-of-cargo-toml-pyproject-toml",className:"group relative heading-with-anchor",get children(){return["Go to Workspace of ",o(e.code,{children:"Cargo.toml"})," / ",o(e.code,{children:"pyproject.toml"})]}}),`
`,o(e.p,{children:"Tombi provides enhanced go to definition features for Rust and Python projects:"}),`
`,o(e.ul,{get children(){return[`
`,o(e.li,{get children(){return["For Rust projects using Cargo, see ",o(e.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo",children:"tombi-extension-cargo"})," for details about go to definition in ",o(e.code,{children:"Cargo.toml"})," files."]}}),`
`,o(e.li,{get children(){return["For Python projects using ",o(e.code,{children:"pyproject.toml"}),", see ",o(e.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject",children:"tombi-extension-pyproject"})," for details about go to definition in ",o(e.code,{children:"pyproject.toml"})," files."]}}),`
`]}})]}function g(t={}){const{wrapper:e}={...r(),...t.components};return e?o(e,c(t,{get children(){return o(n,t)}})):n(t)}function l(t,e){throw new Error("Expected component `"+t+"` to be defined: you likely forgot to import, pass, or provide it.")}export{g as default};
