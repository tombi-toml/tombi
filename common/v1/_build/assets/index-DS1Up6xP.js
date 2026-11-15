import{M as r}from"./index-6OQr06v0.js";import{W as c}from"./Highlight-CMhf5ZLw.js";import{b as n,k as s}from"./web-DCz9_uPG.js";function i(o){const e={a:"a",code:"code",h1:"h1",li:"li",p:"p",ul:"ul",...r(),...o.components},{PageHeading:t}=e;return t||a("PageHeading"),[n(t,{title:"Extensions",description:"Project-specific extensions for Cargo and pyproject.toml.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/extensions"}),`
`,`
`,n(e.h1,{id:"extensions",className:"group relative heading-with-anchor",children:"Extensions"}),`
`,n(c,{get children(){return n(e.p,{children:"The following features are experimental and may change in the future."})}}),`
`,n(e.p,{children:`Tombi currently ships experimental built-in extensions for well-known TOML ecosystems.
These extensions provide project-aware editor features that go beyond plain JSON Schema support.`}),`
`,n(e.p,{children:"Current extension pages:"}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{get children(){return[n(e.a,{href:"/tombi/common/v1/docs/extensions/tombi-extension-cargo",children:"Cargo Extension"})," for Rust ",n(e.code,{children:"Cargo.toml"})]}}),`
`,n(e.li,{get children(){return[n(e.a,{href:"/tombi/common/v1/docs/extensions/tombi-extension-pyproject",children:"Pyproject Extension"})," for Python ",n(e.code,{children:"pyproject.toml"})]}}),`
`]}}),`
`,n(e.p,{children:"Typical extension features include:"}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{children:"workspace-aware go to definition / go to declaration"}),`
`,n(e.li,{children:"package metadata hover"}),`
`,n(e.li,{children:"project-specific completion"}),`
`,n(e.li,{children:"document links to manifests, registries, repositories, and local files"}),`
`,n(e.li,{children:"refactoring-oriented code actions"}),`
`]}}),`
`,n(e.p,{get children(){return["You can also enable or disable individual extension features from ",n(e.a,{href:"/tombi/common/v1/docs/configuration#extensions",get children(){return n(e.code,{children:"[extensions]"})}}),"."]}})]}function p(o={}){const{wrapper:e}={...r(),...o.components};return e?n(e,s(o,{get children(){return n(i,o)}})):i(o)}function a(o,e){throw new Error("Expected component `"+o+"` to be defined: you likely forgot to import, pass, or provide it.")}export{p as default};
