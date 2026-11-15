import{M as l}from"./index-6OQr06v0.js";import{b as e,k as c}from"./web-DCz9_uPG.js";function r(o){const n={code:"code",h1:"h1",h2:"h2",p:"p",...l(),...o.components},{CodeBlock:t,PageHeading:i}=n;return t||a("CodeBlock"),i||a("PageHeading"),[e(i,{title:"Python",description:"Use the tombi-lib package from PyPI.",og_url:"https://tombi-toml.github.io/tombi/main/docs/library/python"}),`
`,e(n.h1,{id:"python",className:"group relative heading-with-anchor",children:"Python"}),`
`,e(n.h2,{id:"install",className:"group relative heading-with-anchor",children:"Install"}),`
`,e(t,{code:"pip install tombi-lib",language:"sh"}),`
`,e(n.h2,{id:"usage",className:"group relative heading-with-anchor",children:"Usage"}),`
`,e(t,{code:`import tombi_lib

result = tombi_lib.format("key=1", "example.toml")
print(result.formatted)  # key = 1

result = tombi_lib.lint("key =", "example.toml")
for diagnostic in result.diagnostics:
    print(diagnostic.level, diagnostic.message)`,language:"python"}),`
`,e(n.p,{get children(){return[e(n.code,{children:"format_async"})," / ",e(n.code,{children:"lint_async"})," are also available for ",e(n.code,{children:"asyncio"}),":"]}}),`
`,e(t,{code:'result = await tombi_lib.format_async("key=1", "example.toml")',language:"python"}),`
`,e(n.h2,{id:"configuration",className:"group relative heading-with-anchor",children:"Configuration"}),`
`,e(n.p,{get children(){return["By default, ",e(n.code,{children:"tombi.toml"}),` is searched from the current directory.
Pass `,e(n.code,{children:"config"})," to use an in-memory configuration instead:"]}}),`
`,e(t,{code:'tombi_lib.format(source, "Cargo.toml", {"config": "[format.rules]\\nindent-width = 4\\n"})',language:"python"})]}function m(o={}){const{wrapper:n}={...l(),...o.components};return n?e(n,c(o,{get children(){return e(r,o)}})):r(o)}function a(o,n){throw new Error("Expected component `"+o+"` to be defined: you likely forgot to import, pass, or provide it.")}export{m as default};
