import{M as l}from"./index-6OQr06v0.js";import{b as e,k as d}from"./web-DCz9_uPG.js";function o(i){const n={a:"a",code:"code",h1:"h1",h2:"h2",h3:"h3",li:"li",p:"p",strong:"strong",ul:"ul",...l(),...i.components},{CodeBlock:t,PageHeading:r}=n;return t||c("CodeBlock"),r||c("PageHeading"),[e(r,{title:"Tombi Document Directive",description:"Control document-level configuration via comments.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/comment-directive/tombi-document-directive"}),`
`,e(n.h1,{id:"tombi-document-directive",className:"group relative heading-with-anchor",children:"Tombi Document Directive"}),`
`,e(n.p,{get children(){return[e(n.code,{children:"#:tombi"})," adds instructions to Tombi. The instructions can be written in the TOML syntax, and can specify the TOML version information, etc."]}}),`
`,e(n.p,{children:"Document comment directives are comments that apply to the entire document. This only works in the comment group at the beginning of the document, so you need to add a blank line before the document value."}),`
`,e(n.p,{get children(){return["You can specify some of these options with the ",e(n.a,{href:"/tombi/common/v1/docs/configuration",children:"Tombi configuration file"}),` as well.
However, settings specified by comment directives always take precedence over those in the configuration file.`]}}),`
`,e(n.h2,{id:"available-options",className:"group relative heading-with-anchor",children:"Available Options"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#toml-version",children:"toml-version"})}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#format",children:"format"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#format-disabled",children:"format.disabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lint",children:"lint"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lint-disabled",children:"lint.disabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schema",children:"schema"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schema-strict",children:"schema.strict"})}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.h3,{id:"toml-version",className:"group relative heading-with-anchor",children:"toml-version"}),`
`,e(n.p,{children:"Specifies the TOML version of the document. This directive has the highest priority."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.strong,{children:"Type"}),": ",e(n.code,{children:"string"})]}}),`
`,e(n.li,{get children(){return[e(n.strong,{children:"Default"}),": ",e(n.code,{children:'"v1.0.0"'})]}}),`
`,e(n.li,{get children(){return[e(n.strong,{children:"Values"}),": ",e(n.code,{children:'"v1.0.0" | "v1.1.0"'})]}}),`
`]}}),`
`,e(t,{code:'#:tombi toml-version = "v1.0.0"',language:"toml"}),`
`,e(n.h3,{id:"format",className:"group relative heading-with-anchor",children:"format"}),`
`,e(n.p,{children:"Formatter options for the document."}),`
`,e(n.h3,{id:"format-disabled",className:"group relative heading-with-anchor",children:"format.disabled"}),`
`,e(n.p,{children:"Disables formatting for the entire document."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.strong,{children:"Type"}),": ",e(n.code,{children:"boolean"})]}}),`
`,e(n.li,{get children(){return[e(n.strong,{children:"Default"}),": ",e(n.code,{children:"false"})]}}),`
`,e(n.li,{get children(){return[e(n.strong,{children:"Values"}),": Only ",e(n.code,{children:"true"})," can be specified"]}}),`
`]}}),`
`,e(t,{code:"#:tombi format.disabled = true",language:"toml"}),`
`,e(n.h3,{id:"lint",className:"group relative heading-with-anchor",children:"lint"}),`
`,e(n.p,{children:"Linter options for the document."}),`
`,e(n.h3,{id:"lint-disabled",className:"group relative heading-with-anchor",children:"lint.disabled"}),`
`,e(n.p,{children:"Disables linting for the entire document."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.strong,{children:"Type"}),": ",e(n.code,{children:"boolean"})]}}),`
`,e(n.li,{get children(){return[e(n.strong,{children:"Default"}),": ",e(n.code,{children:"false"})]}}),`
`,e(n.li,{get children(){return[e(n.strong,{children:"Values"}),": Only ",e(n.code,{children:"true"})," can be specified"]}}),`
`]}}),`
`,e(t,{code:"#:tombi lint.disabled = true",language:"toml"}),`
`,e(n.h3,{id:"schema",className:"group relative heading-with-anchor",children:"schema"}),`
`,e(n.p,{children:"JSON Schema validation options for the document."}),`
`,e(n.h3,{id:"schema-strict",className:"group relative heading-with-anchor",children:"schema.strict"}),`
`,e(n.p,{get children(){return["Enables strict schema validation. If ",e(n.code,{children:"additionalProperties"})," is not specified in the JSON Schema, the strict mode treats it as ",e(n.code,{children:"additionalProperties: false"}),", which is different from the JSON Schema specification."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.strong,{children:"Type"}),": ",e(n.code,{children:"boolean"})]}}),`
`,e(n.li,{get children(){return[e(n.strong,{children:"Default"}),": ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(t,{code:"#:tombi schema.strict = false",language:"toml"})]}function s(i={}){const{wrapper:n}={...l(),...i.components};return n?e(n,d(i,{get children(){return e(o,i)}})):o(i)}function c(i,n){throw new Error("Expected component `"+i+"` to be defined: you likely forgot to import, pass, or provide it.")}export{s as default};
