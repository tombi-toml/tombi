import{b as e,k as c,i as s,t as a}from"./web-DCz9_uPG.js";import{M as r}from"./index-6OQr06v0.js";var d=a('<div class="grid grid-cols-1 md:grid-cols-2 gap-4"><img src=/tombi/common/v1/goto-type-definition.png alt="Go to Type Definition"class=w-full><img src=/tombi/common/v1/goto-type-definition-result.png alt="Go to Type Definition"class=w-full>');function n(t){const o={a:"a",code:"code",h1:"h1",h2:"h2",li:"li",p:"p",ul:"ul",...r(),...t.components},{PageHeading:i}=o;return i||l("PageHeading"),[e(i,{title:"Go to Definition",description:"Navigate to schema definitions and references.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/language-server/goto-definition"}),`
`,e(o.h1,{id:"go-to-definition",className:"group relative heading-with-anchor",children:"Go to Definition"}),`
`,e(o.p,{children:"Tombi supports go to definition."}),`
`,e(o.h2,{id:"go-to-json-schema",className:"group relative heading-with-anchor",children:"Go to JSON Schema"}),`
`,e(o.p,{children:"In TOML files with an applied JSON Schema, 'Go to Type Definition' jumps to the TOML file where the JSON Schema is defined."}),`
`,s(d),`
`,e(o.h2,{id:"go-to-workspace-of-cargo-toml-pyproject-toml",className:"group relative heading-with-anchor",get children(){return["Go to Workspace of ",e(o.code,{children:"Cargo.toml"})," / ",e(o.code,{children:"pyproject.toml"})]}}),`
`,e(o.p,{children:"Tombi provides enhanced go to definition features for Rust and Python projects:"}),`
`,e(o.ul,{get children(){return[`
`,e(o.li,{get children(){return["For Rust projects using Cargo, see ",e(o.a,{href:"/tombi/common/v1/docs/extensions/tombi-extension-cargo",children:"tombi-extension-cargo"})," for details about go to definition in ",e(o.code,{children:"Cargo.toml"})," files."]}}),`
`,e(o.li,{get children(){return["For Python projects using ",e(o.code,{children:"pyproject.toml"}),", see ",e(o.a,{href:"/tombi/common/v1/docs/extensions/tombi-extension-pyproject",children:"tombi-extension-pyproject"})," for details about go to definition in ",e(o.code,{children:"pyproject.toml"})," files."]}}),`
`]}})]}function g(t={}){const{wrapper:o}={...r(),...t.components};return o?e(o,c(t,{get children(){return e(n,t)}})):n(t)}function l(t,o){throw new Error("Expected component `"+t+"` to be defined: you likely forgot to import, pass, or provide it.")}export{g as default};
