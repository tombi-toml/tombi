import{M as i}from"./index-6OQr06v0.js";import{b as t,k as a}from"./web-DCz9_uPG.js";function r(n){const e={a:"a",code:"code",h1:"h1",h2:"h2",li:"li",p:"p",ul:"ul",...i(),...n.components},{PageHeading:o}=e;return o||l("PageHeading"),[t(o,{title:"Overview",description:"a TOML toolkit that provides Formatter, Linter, and Language Server.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs"}),`
`,t(e.h1,{id:"tombi-toml-toolkit",className:"group relative heading-with-anchor",children:"Tombi | TOML Toolkit"}),`
`,t(e.p,{children:"Welcome to the Tombi documentation!"}),`
`,t(e.p,{get children(){return["Tombi (鳶 ",t(e.a,{href:"https://ipa-reader.com/?text=to%C9%B4bi",children:"/toɴbi/"}),") is a powerful toolkit that provides a ",t(e.a,{href:"https://toml.io/",children:"TOML"})," ",t(e.code,{children:"Formatter"}),", ",t(e.code,{children:"Linter"}),`,
and `,t(e.code,{children:"Language Server"})," to help you maintain clean and consistent TOML files."]}}),`
`,t(e.h2,{id:"getting-started",className:"group relative heading-with-anchor",children:"Getting Started"}),`
`,t(e.p,{get children(){return["Visit our ",t(e.a,{href:"/tombi/common/v1/docs/installation",children:"Installation"})," page to begin using Tombi in your projects."]}}),`
`,t(e.h2,{id:"why-tombi",className:"group relative heading-with-anchor",children:"Why Tombi?"}),`
`,t(e.p,{get children(){return["TOML has become a popular configuration format used in many modern development tools, including ",t(e.a,{href:"https://peps.python.org/pep-0621/",get children(){return t(e.code,{children:"pyproject.toml"})}})," for Python and ",t(e.a,{href:"https://doc.rust-lang.org/cargo/reference/manifest.html",get children(){return t(e.code,{children:"Cargo.toml"})}})," for Rust."]}}),`
`,t(e.p,{children:"While TOML offers several advantages over other configuration formats:"}),`
`,t(e.ul,{get children(){return[`
`,t(e.li,{children:"Unlike JSON, it supports comments and has a more human-readable syntax"}),`
`,t(e.li,{children:"Unlike YAML, it has a simpler specification with fewer implicit behaviors"}),`
`]}}),`
`,t(e.p,{get children(){return["However, unlike YAML and JSON, TOML has lacked a reliable and comprehensive tooling solution. While YAML has an excellent ",t(e.a,{href:"https://github.com/redhat-developer/yaml-language-server",children:"Language Server"})," created by RedHat, and JSON has widespread IDE support, TOML developers have been missing similar robust tooling."]}}),`
`,t(e.p,{children:"We created Tombi to support teams that use TOML as their configuration format. Our goal is to provide a reliable and efficient development experience for TOML files."})]}function h(n={}){const{wrapper:e}={...i(),...n.components};return e?t(e,a(n,{get children(){return t(r,n)}})):r(n)}function l(n,e){throw new Error("Expected component `"+n+"` to be defined: you likely forgot to import, pass, or provide it.")}export{h as default};
