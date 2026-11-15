import{M as l}from"./index-6OQr06v0.js";import{b as o,k as d}from"./web-DCz9_uPG.js";function i(n){const e={a:"a",code:"code",h1:"h1",h2:"h2",li:"li",ol:"ol",p:"p",...l(),...n.components},{CodeBlock:t,PageHeading:r}=e;return t||c("CodeBlock"),r||c("PageHeading"),[o(r,{title:"TOML Versions",description:"Supported TOML versions and their differences.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/reference/toml-versions"}),`
`,`
`,o(e.h1,{id:"toml-versions",className:"group relative heading-with-anchor",children:"TOML Versions"}),`
`,o(e.h2,{id:"toml-v1-1-0",className:"group relative heading-with-anchor",children:"TOML v1.1.0"}),`
`,o(e.p,{get children(){return[o(e.a,{href:"https://github.com/toml-lang/toml/releases/tag/1.1.0",children:"TOML v1.1.0"}),` has been released, but many tools do not support it yet.
Therefore, the default TOML version is `,o(e.code,{children:"v1.0.0"}),"."]}}),`
`,o(e.p,{get children(){return["If you want to use ",o(e.code,{children:"v1.1.0"}),", please refer to the next section."]}}),`
`,o(e.h2,{id:"toml-version-control",className:"group relative heading-with-anchor",children:"TOML Version Control"}),`
`,o(e.p,{children:"Tombi determines the TOML version for the target file in the following order of priority."}),`
`,o(e.ol,{get children(){return[`
`,o(e.li,{get children(){return["The comment directive ",o(e.code,{children:'#:tombi toml-version = "v1.1.0"'})," in the TOML document."]}}),`
`,o(e.li,{get children(){return[o(e.code,{children:"toml-version"})," specified in ",o(e.code,{children:"[[schemas]]"})," in the Tombi config."]}}),`
`,o(e.li,{get children(){return[o(e.code,{children:"x-tombi-toml-version"})," in the root object of JSON Schema."]}}),`
`,o(e.li,{get children(){return[o(e.code,{children:"toml-version"})," specified in the Tombi config."]}}),`
`,o(e.li,{get children(){return["The default TOML version (Currently ",o(e.code,{children:"v1.0.0"}),")."]}}),`
`]}}),`
`,o(e.p,{get children(){return["To specify the TOML version at the configuration-file level (priority 4), set ",o(e.code,{children:"toml-version"})," at the root of the Tombi config."]}}),`
`,o(t,{code:'toml-version = "v1.0.0"',language:"toml"}),`
`,o(e.p,{get children(){return["To override the version declared by a JSON Schema for matching files (priority 3), set ",o(e.code,{children:"toml-version"})," in ",o(e.code,{children:"[[schemas]]"}),"."]}}),`
`,o(t,{code:`[[schemas]]
toml-version = "v1.0.0"
path = "tombi://www.schemastore.org/cargo.json"
include = ["Cargo.toml"]`,language:"toml"})]}function a(n={}){const{wrapper:e}={...l(),...n.components};return e?o(e,d(n,{get children(){return o(i,n)}})):i(n)}function c(n,e){throw new Error("Expected component `"+n+"` to be defined: you likely forgot to import, pass, or provide it.")}export{a as default};
