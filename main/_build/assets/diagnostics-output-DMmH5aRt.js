import{M as c}from"./index-6OQr06v0.js";import{b as t,k as l}from"./web-DCz9_uPG.js";function o(n){const e={a:"a",code:"code",h1:"h1",h2:"h2",li:"li",p:"p",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",ul:"ul",...c(),...n.components},{CodeBlock:r,PageHeading:i}=e;return r||d("CodeBlock"),i||d("PageHeading"),[t(i,{title:"Diagnostics Output",description:"Write diagnostics in JSON, GitHub, GitLab, JUnit, and SARIF formats.",og_url:"https://tombi-toml.github.io/tombi/main/docs/cli/diagnostics-output"}),`
`,t(e.h1,{id:"diagnostics-output",className:"group relative heading-with-anchor",children:"Diagnostics Output"}),`
`,t(e.p,{get children(){return[t(e.code,{children:"tombi format"})," and ",t(e.code,{children:"tombi lint"})," can write diagnostics in machine-readable formats for CI and other tools."]}}),`
`,t(r,{code:`# JUnit report of unformatted files
tombi format --check --diagnostics-format junit --diagnostics-file junit.xml

# GitLab Code Quality report
tombi lint --diagnostics-format gitlab --diagnostics-file gl-code-quality-report.json

# GitHub Actions annotations
tombi lint --diagnostics-format github

# JSON to stdout
tombi lint --quiet --diagnostics-format json --diagnostics-file /dev/stdout | jq`,language:"bash"}),`
`,t(e.h2,{id:"options",className:"group relative heading-with-anchor",children:"Options"}),`
`,t(e.ul,{get children(){return[`
`,t(e.li,{get children(){return[t(e.code,{children:"--diagnostics-format <FORMAT>"}),": Output format of diagnostics. The default is ",t(e.code,{children:"pretty"}),"."]}}),`
`,t(e.li,{get children(){return[t(e.code,{children:"--diagnostics-file <PATH>"}),`: File to write diagnostics to. The default is stderr.
On Unix, `,t(e.code,{children:"/dev/stdout"})," and ",t(e.code,{children:"/dev/fd/N"})," are also supported."]}}),`
`]}}),`
`,t(e.h2,{id:"formats",className:"group relative heading-with-anchor",children:"Formats"}),`
`,t(e.table,{get children(){return[t(e.thead,{get children(){return t(e.tr,{get children(){return[t(e.th,{children:"Format"}),t(e.th,{children:"Description"}),t(e.th,{style:{textAlign:"center"},get children(){return["Requires ",t(e.code,{children:"--diagnostics-file"})]}})]}})}}),t(e.tbody,{get children(){return[t(e.tr,{get children(){return[t(e.td,{get children(){return t(e.code,{children:"pretty"})}}),t(e.td,{children:"Human-readable output"}),t(e.td,{style:{textAlign:"center"}})]}}),t(e.tr,{get children(){return[t(e.td,{get children(){return t(e.code,{children:"json"})}}),t(e.td,{children:"JSON array of diagnostics"}),t(e.td,{style:{textAlign:"center"},children:"✅"})]}}),t(e.tr,{get children(){return[t(e.td,{get children(){return t(e.code,{children:"json-lines"})}}),t(e.td,{get children(){return[t(e.a,{href:"https://jsonlines.org/",children:"JSON Lines"}),", a diagnostic per line"]}}),t(e.td,{style:{textAlign:"center"},children:"✅"})]}}),t(e.tr,{get children(){return[t(e.td,{get children(){return t(e.code,{children:"github"})}}),t(e.td,{get children(){return t(e.a,{href:"https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands",children:"GitHub Actions workflow commands"})}}),t(e.td,{style:{textAlign:"center"}})]}}),t(e.tr,{get children(){return[t(e.td,{get children(){return t(e.code,{children:"gitlab"})}}),t(e.td,{get children(){return t(e.a,{href:"https://docs.gitlab.com/ci/testing/code_quality/",children:"GitLab Code Quality report"})}}),t(e.td,{style:{textAlign:"center"},children:"✅"})]}}),t(e.tr,{get children(){return[t(e.td,{get children(){return t(e.code,{children:"junit"})}}),t(e.td,{children:"JUnit XML report"}),t(e.td,{style:{textAlign:"center"},children:"✅"})]}}),t(e.tr,{get children(){return[t(e.td,{get children(){return t(e.code,{children:"sarif"})}}),t(e.td,{get children(){return[t(e.a,{href:"https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html",children:"SARIF 2.1.0"})," report"]}}),t(e.td,{style:{textAlign:"center"},children:"✅"})]}})]}})]}}),`
`,t(e.p,{get children(){return[`Lines and columns are 1-based.
Columns count grapheme clusters, like the output of `,t(e.code,{children:"pretty"}),", except for ",t(e.code,{children:"sarif"}),", which counts UTF-16 code units."]}}),`
`,t(e.p,{children:"Problems with a whole file are reported with the following codes:"}),`
`,t(e.ul,{get children(){return[`
`,t(e.li,{get children(){return[t(e.code,{children:"not-formatted"}),": the file is not formatted (",t(e.code,{children:"tombi format --check"}),")."]}}),`
`,t(e.li,{get children(){return[t(e.code,{children:"io-error"}),": the file could not be found, read, or written."]}}),`
`]}}),`
`,t(e.p,{get children(){return["When reading from stdin, formats other than ",t(e.code,{children:"pretty"})," require ",t(e.code,{children:"--stdin-filename"}),"."]}})]}function s(n={}){const{wrapper:e}={...c(),...n.components};return e?t(e,l(n,{get children(){return t(o,n)}})):o(n)}function d(n,e){throw new Error("Expected component `"+n+"` to be defined: you likely forgot to import, pass, or provide it.")}export{s as default};
