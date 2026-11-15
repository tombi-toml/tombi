import{M as c}from"./index-6OQr06v0.js";import{b as e,k as m}from"./web-DCz9_uPG.js";function r(t){const n={a:"a",br:"br",code:"code",h1:"h1",h2:"h2",p:"p",...c(),...t.components},{CodeBlock:a,PageHeading:i}=n;return a||o("CodeBlock"),i||o("PageHeading"),[e(i,{title:"Magic Trailing Comma",description:"black-inspired formatting with magic trailing comma.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/formatter/magic-trailing-comma"}),`
`,e(n.h1,{id:"magic-trailing-comma",className:"group relative heading-with-anchor",children:"Magic Trailing Comma"}),`
`,e(n.p,{get children(){return["Magic Trailing Comma is a feature inspired by the ",e(n.a,{href:"https://black.readthedocs.io/en/stable/the_black_code_style/current_style.html#the-magic-trailing-comma",children:"Black formatter"}),".",e(n.br,{}),`
`,"When you add a trailing comma to the last element, Tombi treats it as an explicit hint to expand into multiple lines and keeps the trailing comma after formatting.",e(n.br,{}),`
`,"This keeps diffs clean when new elements are added later, while respecting the author's intent when no trailing comma is present."]}}),`
`,e(n.h2,{id:"basic-usage",className:"group relative heading-with-anchor",children:"Basic Usage"}),`
`,e(n.p,{children:"When there is no trailing comma, Tombi automatically folds the array:"}),`
`,e(a,{code:`# Before formatting
items = [
  "aaaa",
  "bbbb",
  "cccc"
]

# After formatting
items = ["aaaa", "bbbb", "cccc"]`,language:"toml"}),`
`,e(n.p,{get children(){return["If ",e(n.code,{children:"line-width"})," is configured and the array exceeds that width, the array will be expanded to multiple lines, but the final element does not receive a trailing comma automatically. When ",e(n.code,{children:"line-width"})," is omitted, arrays are not expanded based on line width."]}}),`
`,e(a,{code:`# Before formatting
items = ["aaaa", "bbbb", "cccc", "dddd", "eeee", "ffff", "gggg", "hhhh", "iiii","jjjj"]

# After formatting
items = [
  "aaaa",
  "bbbb",
  "cccc",
  "dddd",
  "eeee",
  "ffff",
  "gggg",
  "hhhh",
  "iiii",
  "jjjj"
]`,language:"toml"}),`
`,e(n.p,{get children(){return["Alternatively, you can force it to expand to multiple lines by adding a comma to the last element of the array.",e(n.br,{}),`
`,"The formatter preserves that trailing comma in the expanded form."]}}),`
`,e(a,{code:`# Before formatting
dependencies = ["tombi", "biome", "prettier",]

# After formatting
dependencies = [
  "tombi",
  "biome",
  "prettier",
]`,language:"toml"}),`
`,e(n.h2,{id:"magic-trailing-comma-for-inline-tables",className:"group relative heading-with-anchor",children:"Magic Trailing Comma for Inline Tables"}),`
`,e(n.p,{children:"In TOML v1.1.0, Magic Trailing Comma will also be enabled for Inline Tables, and the same trailing-comma hint applies."}),`
`,e(a,{code:`# Before formatting
items = { a = 1, b = 2, c = 3, }

# After formatting
items = {
  a = 1,
  b = 2,
  c = 3,
}`,language:"toml"})]}function g(t={}){const{wrapper:n}={...c(),...t.components};return n?e(n,m(t,{get children(){return e(r,t)}})):r(t)}function o(t,n){throw new Error("Expected component `"+t+"` to be defined: you likely forgot to import, pass, or provide it.")}export{g as default};
