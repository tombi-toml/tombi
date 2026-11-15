import{M as o}from"./index-6OQr06v0.js";import{b as t,k as c}from"./web-DCz9_uPG.js";function r(n){const e={a:"a",code:"code",h1:"h1",h3:"h3",li:"li",p:"p",strong:"strong",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",ul:"ul",...o(),...n.components},{PageHeading:i}=e;return i||d("PageHeading"),[t(i,{title:"Comment Directive",description:"Control formatting and schema using special comments.",og_url:"https://tombi-toml.github.io/tombi/main/docs/comment-directive"}),`
`,`
`,t(e.h1,{id:"comment-directive",className:"group relative heading-with-anchor",children:"Comment Directive"}),`
`,t(e.p,{children:"Tombi supports comment directives to control the behavior of the document."}),`
`,t(e.p,{children:"There are two types of comment directives with different scopes of application:"}),`
`,t(e.table,{get children(){return[t(e.thead,{get children(){return t(e.tr,{get children(){return[t(e.th,{children:"Type"}),t(e.th,{children:"Directive"}),t(e.th,{children:"Scope"})]}})}}),t(e.tbody,{get children(){return[t(e.tr,{get children(){return[t(e.td,{children:"Document Comment Directive"}),t(e.td,{get children(){return[t(e.code,{children:"#:schema"}),", ",t(e.code,{children:"#:tombi"})]}}),t(e.td,{children:"Document"})]}}),t(e.tr,{get children(){return[t(e.td,{children:"Value Comment Directive"}),t(e.td,{get children(){return t(e.code,{children:"# tombi:"})}}),t(e.td,{children:"Value"})]}})]}})]}}),`
`,t(e.h3,{id:"document-comment-directive",className:"group relative heading-with-anchor",children:"Document Comment Directive"}),`
`,t(e.p,{children:"Document comment directives are comments that apply to the entire document. These only work in the comment group at the beginning of the document."}),`
`,t(e.ul,{get children(){return[`
`,t(e.li,{get children(){return[t(e.strong,{get children(){return t(e.a,{href:"/tombi/main/docs/comment-directive/schema-document-directive",children:"#:schema"})}})," - Specify JSON Schema for validation"]}}),`
`,t(e.li,{get children(){return[t(e.strong,{get children(){return t(e.a,{href:"/tombi/main/docs/comment-directive/tombi-document-directive",children:"#:tombi"})}})," - Document-level configuration (TOML version, format/lint/schema options)"]}}),`
`]}}),`
`,t(e.h3,{id:"value-comment-directive",className:"group relative heading-with-anchor",children:"Value Comment Directive"}),`
`,t(e.p,{children:"Value comment directives are comments that apply to a specific value, allowing fine-grained control over formatting and linting rules."}),`
`,t(e.ul,{get children(){return[`
`,t(e.li,{get children(){return[t(e.strong,{get children(){return t(e.a,{href:"/tombi/main/docs/comment-directive/tombi-value-directive",children:"# tombi:"})}})," - Value-level configuration (format rules, lint rules)"]}}),`
`]}})]}function h(n={}){const{wrapper:e}={...o(),...n.components};return e?t(e,c(n,{get children(){return t(r,n)}})):r(n)}function d(n,e){throw new Error("Expected component `"+n+"` to be defined: you likely forgot to import, pass, or provide it.")}export{h as default};
