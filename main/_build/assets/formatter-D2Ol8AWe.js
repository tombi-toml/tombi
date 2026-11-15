import{M as l}from"./index-6OQr06v0.js";import{b as e,k as c}from"./web-DCz9_uPG.js";function m(o){const t={a:"a",br:"br",code:"code",h1:"h1",p:"p",...l(),...o.components},{CodeBlock:r,Note:i,PageHeading:a}=t;return r||n("CodeBlock"),i||n("Note"),a||n("PageHeading"),[e(a,{title:"Formatter",description:"Format TOML files from the command line.",og_url:"https://tombi-toml.github.io/tombi/main/docs/cli/formatter"}),`
`,e(t.h1,{id:"formatter-cli-usage",className:"group relative heading-with-anchor",children:"Formatter CLI Usage"}),`
`,e(t.p,{children:"Tombi's formatter can be run via the CLI."}),`
`,e(r,{code:`# If you have \`tombi.toml\` in your project, Format all TOML files in the project,
# if not, it will format all TOML files in the current directory
tombi format

# Format a single file
tombi format path/to/file.toml

# Format multiple files
tombi format path/to/file1.toml path/to/file2.toml

# Format all TOML files in a directory
tombi format .

# Format files using glob pattern
tombi format **/*.toml

# Format from standard input
cat Cargo.toml | tombi format -`,language:"bash"}),`
`,e(i,{get children(){return e(t.p,{get children(){return["If only warnings are logged during formatting, such as a schema that could not be fetched, Tombi will exit successfully by default.",e(t.br,{}),`
`,"However, you can use the ",e(t.code,{children:"--error-on-warnings"})," option to make Tombi exit with an error when warnings are present."]}})}}),`
`,e(t.p,{get children(){return["See ",e(t.a,{href:"/tombi/main/docs/cli/diagnostics-output",children:"Diagnostics Output"})," to report unformatted files in machine-readable formats such as JUnit and SARIF."]}})]}function d(o={}){const{wrapper:t}={...l(),...o.components};return t?e(t,c(o,{get children(){return e(m,o)}})):m(o)}function n(o,t){throw new Error("Expected component `"+o+"` to be defined: you likely forgot to import, pass, or provide it.")}export{d as default};
