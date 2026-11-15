import{M as a}from"./index-6OQr06v0.js";import{N as c}from"./Highlight-CMhf5ZLw.js";import{b as t,k as s}from"./web-DCz9_uPG.js";function r(e){const n={a:"a",br:"br",code:"code",h1:"h1",p:"p",...a(),...e.components},{CodeBlock:i,PageHeading:o}=n;return i||l("CodeBlock"),o||l("PageHeading"),[t(o,{title:"Linter",description:"Lint TOML files from the command line.",og_url:"https://tombi-toml.github.io/tombi/main/docs/cli/linter"}),`
`,`
`,t(n.h1,{id:"linter-cli-usage",className:"group relative heading-with-anchor",children:"Linter CLI Usage"}),`
`,t(n.p,{children:"Tombi's linter can be run via the CLI."}),`
`,t(i,{code:`# If you have \`tombi.toml\` in your project, Lint all TOML files in the project,
# if not, it will lint all TOML files in the current directory
tombi lint

# Lint a single file
tombi lint path/to/file.toml

# Lint multiple files
tombi lint path/to/file1.toml path/to/file2.toml

# Lint all TOML files in a directory
tombi lint .

# Lint files using glob pattern
tombi lint **/*.toml

# Lint from standard input
cat Cargo.toml | tombi lint -`,language:"bash"}),`
`,t(c,{get children(){return t(n.p,{get children(){return["If only warnings are found during linting, Tombi will exit successfully by default.",t(n.br,{}),`
`,"However, you can use the ",t(n.code,{children:"--error-on-warnings"})," option to make Tombi exit with an error when warnings are present, including warnings that are logged, such as a schema that could not be fetched."]}})}}),`
`,t(n.p,{get children(){return["See ",t(n.a,{href:"/tombi/main/docs/cli/diagnostics-output",children:"Diagnostics Output"})," for machine-readable formats such as GitLab Code Quality and SARIF."]}})]}function h(e={}){const{wrapper:n}={...a(),...e.components};return n?t(n,s(e,{get children(){return t(r,e)}})):r(e)}function l(e,n){throw new Error("Expected component `"+e+"` to be defined: you likely forgot to import, pass, or provide it.")}export{h as default};
