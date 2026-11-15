import{M as l}from"./index-6OQr06v0.js";import{b as e,k as c}from"./web-DCz9_uPG.js";function o(t){const n={a:"a",br:"br",code:"code",h1:"h1",h2:"h2",h3:"h3",li:"li",p:"p",strong:"strong",ul:"ul",...l(),...t.components},{CodeBlock:r,PageHeading:a}=n;return r||i("CodeBlock"),a||i("PageHeading"),[e(a,{title:"Treatment of Comments",description:"How Tombi handles comments during formatting.",og_url:"https://tombi-toml.github.io/tombi/main/docs/formatter/treatment-of-comments"}),`
`,e(n.h1,{id:"treatment-of-comments",className:"group relative heading-with-anchor",children:"Treatment of Comments"}),`
`,e(n.p,{children:"Tombi uses a group-based comment model to keep formatting stable and auto sorting predictable."}),`
`,e(n.p,{children:"This page explains the behavior in plain TOML examples, without implementation details."}),`
`,e(n.h2,{id:"quick-summary",className:"group relative heading-with-anchor",children:"Quick Summary"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{children:"Tombi treats blank-line-separated comment blocks as meaningful boundaries."}),`
`,e(n.li,{children:"Sorting happens inside each group, not across group boundaries."}),`
`,e(n.li,{get children(){return[e(n.code,{children:"# tombi:"})," directives only control formatter/linter rules when placed in a position that has a clear target."]}}),`
`]}}),`
`,e(n.h2,{id:"comment-roles-in-toml",className:"group relative heading-with-anchor",children:"Comment Roles in TOML"}),`
`,e(n.h3,{id:"leading-comments",className:"group relative heading-with-anchor",children:"Leading comments"}),`
`,e(n.p,{children:"Comments directly attached to the item below."}),`
`,e(r,{code:`# leading comment 1
# leading comment 2
key = "value"`,language:"toml"}),`
`,e(n.h3,{id:"trailing-comments",className:"group relative heading-with-anchor",children:"Trailing comments"}),`
`,e(n.p,{children:"A comment at the end of the same line."}),`
`,e(r,{code:'key = "value" # trailing comment',language:"toml"}),`
`,e(n.h3,{id:"dangling-comment-groups",className:"group relative heading-with-anchor",children:"Dangling comment groups"}),`
`,e(n.p,{children:"Comment blocks separated by blank lines. These create visual/logical boundaries."}),`
`,e(r,{code:`# dangling comment group A line 1
# dangling comment group A line 2

# dangling comment group B line 1
# dangling comment group B line 2`,language:"toml"}),`
`,e(n.h2,{id:"blank-lines-define-boundaries",className:"group relative heading-with-anchor",children:"Blank Lines Define Boundaries"}),`
`,e(n.p,{children:"A blank line is treated as a boundary between groups."}),`
`,e(n.p,{children:"That boundary is important because Tombi does not move values/keys across it when sorting."}),`
`,e(r,{code:`[tool.demo]
b = 2
a = 1

z = 26
y = 25`,language:"toml"}),`
`,e(n.p,{get children(){return["Tombi may reorder ",e(n.code,{children:"b/a"})," and ",e(n.code,{children:"z/y"})," inside each group, but it does not merge or cross the boundary."]}}),`
`,e(n.h2,{id:"where-tombi-directives-are-effective",className:"group relative heading-with-anchor",get children(){return["Where ",e(n.code,{children:"# tombi:"})," Directives Are Effective"]}}),`
`,e(n.p,{children:"A simple rule:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{children:"Effective: before the first item of the current container."}),`
`,e(n.li,{children:"Not effective for rule control: between groups after items already appeared."}),`
`]}}),`
`,e(n.p,{children:"Below are concrete examples."}),`
`,e(n.h3,{id:"root-level-example",className:"group relative heading-with-anchor",children:"Root-level example"}),`
`,e(n.p,{children:"Effective at root level (before first key):"}),`
`,e(r,{code:`# root directive: effective for the root table
# tombi: format.rules.table-keys-order = "ascending"

b = 2
a = 1`,language:"toml"}),`
`,e(n.p,{children:"Not effective at root level (appears after root items started):"}),`
`,e(r,{code:`b = 2
a = 1

# root group-boundary directive:
# interpreted as an empty key-value group directive
# (not used for rule control)
# tombi: format.rules.table-keys-order = "ascending"

z = 26
y = 25`,language:"toml"}),`
`,e(n.p,{get children(){return["In this position, Tombi interprets the directive as belonging to an ",e(n.strong,{children:"empty key-value group"})," at the boundary.",e(n.br,{}),`
`,"So it is recognized by tooling, but it does not affect formatter/linter rule control."]}}),`
`,e(n.h3,{id:"standard-table-example",className:"group relative heading-with-anchor",children:"Standard table example"}),`
`,e(n.p,{get children(){return["Effective for ",e(n.code,{children:"[package]"})," (before first key in that table):"]}}),`
`,e(r,{code:`[package]
# package directive: effective for this table
# tombi: format.rules.table-keys-order = "ascending"

name = "demo"
version = "1.0.0"`,language:"toml"}),`
`,e(n.p,{get children(){return["Not effective for ",e(n.code,{children:"[package]"})," (between table groups after keys already appeared):"]}}),`
`,e(r,{code:`[package]
name = "demo"
version = "1.0.0"

# package group-boundary directive: not used for rule control
# tombi: format.rules.table-keys-order = "ascending"

description = "example"
license = "MIT"`,language:"toml"}),`
`,e(n.h3,{id:"array-example",className:"group relative heading-with-anchor",children:"Array example"}),`
`,e(n.p,{children:"Effective for the array (before first value):"}),`
`,e(r,{code:`items = [
  # array directive: effective for this array
  # tombi: format.rules.array-values-order = "ascending"

  "banana",
  "apple",
]`,language:"toml"}),`
`,e(n.p,{children:"Not effective for the array (between value groups after values already appeared):"}),`
`,e(r,{code:`items = [
  "banana",
  "apple",

  # array group-boundary directive: not used for rule control
  # tombi: format.rules.array-values-order = "ascending"

  "date",
  "cherry",
]`,language:"toml"}),`
`,e(n.h3,{id:"inline-table-example",className:"group relative heading-with-anchor",children:"Inline table example"}),`
`,e(n.p,{children:"Effective for the inline table (before first key-value in the inline table):"}),`
`,e(r,{code:`point = {
  # inline-table directive: effective for this inline table
  # tombi: format.rules.table-keys-order = "ascending"

  y = 20,
  x = 10,
}`,language:"toml"}),`
`,e(n.p,{children:"Not effective for the inline table (between groups after key-values already appeared):"}),`
`,e(r,{code:`point = {
  y = 20,
  x = 10,

  # inline-table group-boundary directive: not used for rule control
  # tombi: format.rules.table-keys-order = "ascending"

  z = 30,
}`,language:"toml"}),`
`,e(n.h3,{id:"array-of-tables-example",className:"group relative heading-with-anchor",children:"Array-of-tables example"}),`
`,e(n.p,{get children(){return["Effective for each ",e(n.code,{children:"[[servers]]"})," block only when placed before that block's first key:"]}}),`
`,e(r,{code:`[[servers]]
# array-of-table directive: effective for this table item
# tombi: format.rules.table-keys-order = "ascending"

port = 8080
host = "localhost"`,language:"toml"}),`
`,e(n.p,{children:"Not effective when placed between groups after keys already appeared:"}),`
`,e(r,{code:`[[servers]]
port = 8080
host = "localhost"

# array-of-table group-boundary directive: not used for rule control
# tombi: format.rules.table-keys-order = "ascending"

protocol = "http"`,language:"toml"}),`
`,e(n.h2,{id:"what-happens-to-non-effective-directives",className:"group relative heading-with-anchor",children:"What Happens to Non-Effective Directives?"}),`
`,e(n.p,{children:"When a directive is placed at a group boundary (not rule-effective):"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{children:"Formatter rule control: no available rules in this position"}),`
`,e(n.li,{children:"Linter rule control: no available rules in this position"}),`
`,e(n.li,{children:"Editor support (completion/hover/validation): still available"}),`
`]}}),`
`,e(n.p,{children:"So you still get language-server assistance, but the directive does not change formatting/linting behavior."}),`
`,e(n.h2,{id:"one-annotated-walkthrough",className:"group relative heading-with-anchor",children:"One Annotated Walkthrough"}),`
`,e(r,{code:`# [root directive: effective for root]
# tombi: format.rules.table-keys-order = "ascending"

# [root dangling comment group]

[package]
# [table directive: effective for package table]
# tombi: format.rules.table-keys-order = "ascending"

name = "demo"
version = "1.0.0"

# [table group-boundary directive: not effective for rule control]
# tombi: format.rules.table-keys-order = "descending"

description = "example"

items = [
  # [array directive: effective for this array]
  # tombi: format.rules.array-values-order = "ascending"

  "b",
  "a",

  # [array group-boundary directive: not effective for rule control]
  # tombi: format.rules.array-values-order = "descending"

  "d",
  "c",
]`,language:"toml"}),`
`,e(n.p,{children:"Reading rule:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{children:'Comments labeled "directive: effective" are used for rule control.'}),`
`,e(n.li,{children:'Comments labeled "group-boundary directive" are treated as boundary comments and are not used for formatter/linter rule control.'}),`
`]}}),`
`,e(n.p,{get children(){return["For directive syntax and available options, see ",e(n.a,{href:"/tombi/main/docs/comment-directive/",children:"Comment Directive"}),"."]}})]}function m(t={}){const{wrapper:n}={...l(),...t.components};return n?e(n,c(t,{get children(){return e(o,t)}})):o(t)}function i(t,n){throw new Error("Expected component `"+t+"` to be defined: you likely forgot to import, pass, or provide it.")}export{m as default};
