import{M as r}from"./index-6OQr06v0.js";import{N as h}from"./Highlight-CMhf5ZLw.js";import{b as e,k as l}from"./web-DCz9_uPG.js";function c(n){const t={a:"a",code:"code",h1:"h1",h2:"h2",h3:"h3",li:"li",p:"p",ul:"ul",...r(),...n.components},{CodeBlock:i,PageHeading:o}=t;return i||a("CodeBlock"),o||a("PageHeading"),[e(o,{title:"Schema Document Directive",description:"Select a JSON Schema for a document with a comment directive.",og_url:"https://tombi-toml.github.io/tombi/main/docs/comment-directive/schema-document-directive"}),`
`,`
`,e(t.h1,{id:"schema-document-directive",className:"group relative heading-with-anchor",children:"Schema Document Directive"}),`
`,e(t.p,{get children(){return["Same as ",e(t.a,{href:"https://taplo.tamasfe.dev/configuration/directives.html#the-schema-directive",children:"Taplo"}),", ",e(t.code,{children:"#:schema"}),` is used to specify the schema to use for the document.
However, Tombi only allows document comment directives at the beginning of the document, so you need to add a blank line after the directive.`]}}),`
`,e(t.h2,{id:"usage",className:"group relative heading-with-anchor",children:"Usage"}),`
`,e(t.h3,{id:"http-https-urls",className:"group relative heading-with-anchor",children:"HTTP/HTTPS URLs"}),`
`,e(i,{code:`#:schema https://example.com/my-schema.json

[config]
value = "example"`,language:"toml"}),`
`,e(t.h3,{id:"local-file-paths",className:"group relative heading-with-anchor",children:"Local File Paths"}),`
`,e(i,{code:`#:schema ./schemas/my-schema.json

[settings]
enabled = true`,language:"toml"}),`
`,e(h,{get children(){return[e(t.p,{get children(){return["Although not defined by ",e(t.a,{href:"https://www.rfc-editor.org/rfc/rfc8089",children:"RFC 8089"}),`,
which specifies the `,e(t.code,{children:"file"}),` URI scheme,
Tombi allows the specification of `,e(t.code,{children:"."})," and ",e(t.code,{children:".."})," in the hostname of the ",e(t.code,{children:"file://"})," file URI."]}}),e(t.p,{children:"These hostname values are resolved as follows:"}),e(t.ul,{get children(){return[`
`,e(t.li,{get children(){return["If the file URI authority is ",e(t.code,{children:"."}),", Tombi treats it as the directory that contains the TOML file."]}}),`
`,e(t.li,{get children(){return["If the file URI authority is ",e(t.code,{children:".."}),", Tombi treats it as the parent directory of the TOML file."]}}),`
`]}}),e(t.p,{children:"This allows you to specify local files in the following ways:"}),e(i,{code:`#:schema file://../../schemas/my-schema.json#/definitions/User

name = "Taro"
country = "Japan"`,language:"toml"})]}})]}function u(n={}){const{wrapper:t}={...r(),...n.components};return t?e(t,l(n,{get children(){return e(c,n)}})):c(n)}function a(n,t){throw new Error("Expected component `"+n+"` to be defined: you likely forgot to import, pass, or provide it.")}export{u as default};
