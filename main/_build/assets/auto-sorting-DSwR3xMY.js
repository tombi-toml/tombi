import{M as s}from"./index-6OQr06v0.js";import{W as c}from"./Highlight-CMhf5ZLw.js";import{b as e,k as d}from"./web-DCz9_uPG.js";function i(o){const r={a:"a",code:"code",h1:"h1",h2:"h2",h3:"h3",li:"li",p:"p",ul:"ul",...s(),...o.components},{CodeBlock:t,PageHeading:n}=r;return t||a("CodeBlock"),n||a("PageHeading"),[e(n,{title:"Auto Sorting",description:"Automatically sort keys using JSON Schema metadata.",og_url:"https://tombi-toml.github.io/tombi/main/docs/formatter/auto-sorting"}),`
`,`
`,e(r.h1,{id:"auto-sorting",className:"group relative heading-with-anchor",children:"Auto Sorting"}),`
`,e(r.p,{children:"One of Tombi's most distinctive features is automatic sorting."}),`
`,e(r.p,{get children(){return[e(r.a,{href:"https://github.com/tamasfe/taplo",children:"Taplo"})," and ",e(r.a,{href:"https://github.com/pappasam/toml-sort",children:"toml-sort"}),` specify one sorting method and apply it consistently across a document.
Tombi is designed differently: shared schema information specifies the sorting method for each item, so those fine-grained sorting policies are shared with everyone who uses the schema.`]}}),`
`,e(r.p,{get children(){return["Also, it can automatically sort content including comments, using its ",e(r.a,{href:"/tombi/main/docs/formatter/treatment-of-comments",children:"own unique treatment of comments"}),"."]}}),`
`,e(r.h2,{id:"how-to-specify-auto-sorting-methods",className:"group relative heading-with-anchor",children:"How to Specify Auto Sorting Methods"}),`
`,e(r.p,{children:"Tombi does not perform automatic sorting unless a sorting method is specified."}),`
`,e(r.p,{children:"Sorting methods are available in the following order of priority:"}),`
`,e(r.ul,{get children(){return[`
`,e(r.li,{get children(){return e(r.a,{href:"/tombi/main/docs/formatter/auto-sorting#comment-directives",children:"Comment Directives"})}}),`
`,e(r.li,{get children(){return e(r.a,{href:"/tombi/main/docs/formatter/auto-sorting#configuration",children:"Configuration"})}}),`
`,e(r.li,{get children(){return e(r.a,{href:"/tombi/main/docs/formatter/auto-sorting#json-schema",children:"JSON Schema"})}}),`
`]}}),`
`,e(r.h3,{id:"comment-directives",className:"group relative heading-with-anchor",children:"Comment Directives"}),`
`,e(r.p,{get children(){return["You can specify auto sorting methods using ",e(r.a,{href:"/tombi/main/docs/comment-directive/tombi-value-directive#format-rules",children:"comment directives"}),`.
This has higher priority than `,e(r.a,{href:"/tombi/main/docs/formatter/auto-sorting#configuration",children:"Configuration"})," and ",e(r.a,{href:"/tombi/main/docs/json-schema#formatting",children:"JSON Schema"})," specifications."]}}),`
`,e(c,{get children(){return e(r.p,{get children(){return["For files that have a schema assigned, the same can also be specified in ",e(r.code,{children:"tombi.toml"})," with ",e(r.a,{href:"/tombi/main/docs/formatter/auto-sorting#configuration",children:"Configuration"}),`.
Use comment directives for temporary overrides, such as working around errors in JSON Schemas registered in the Schema Store,
or for local adjustments in files that have no schema.`]}})}}),`
`,e(r.h3,{id:"configuration",className:"group relative heading-with-anchor",children:"Configuration"}),`
`,e(r.p,{get children(){return["You can specify auto sorting methods in ",e(r.code,{children:"tombi.toml"})," using ",e(r.a,{href:"/tombi/main/docs/configuration#schemas-overrides",get children(){return e(r.code,{children:"schemas[*].overrides[*]"})}}),`.
This has lower priority than `,e(r.a,{href:"/tombi/main/docs/formatter/auto-sorting#comment-directives",children:"Comment Directives"})," and higher priority than ",e(r.a,{href:"/tombi/main/docs/formatter/auto-sorting#json-schema",children:"JSON Schema"})," specifications."]}}),`
`,e(r.p,{get children(){return[`This is the recommended way to set a stable rule for the whole project.
You do not need to edit the JSON Schema or add `,e(r.code,{children:"x-tombi-*"})," keys to it."]}}),`
`,e(r.p,{get children(){return["For example, the following configuration sorts the root table keys of ",e(r.code,{children:"example.toml"})," in ascending order and the ",e(r.code,{children:"items"})," array values in descending order."]}}),`
`,e(t,{code:`[[schemas]]
path = "https://example.com/schema.json"
include = ["example.toml"]

[[schemas.overrides]]
targets = [""]
format.rules.table-keys-order = "ascending"

[[schemas.overrides]]
targets = ["items"]
format.rules.array-values-order = "descending"`,language:"toml"}),`
`,e(r.p,{get children(){return["See ",e(r.a,{href:"/tombi/main/docs/configuration#schemas-overrides-format-rules",get children(){return e(r.code,{children:"schemas[*].overrides[*].format.rules"})}})," for the available targets and values."]}}),`
`,e(r.h3,{id:"json-schema",className:"group relative heading-with-anchor",children:"JSON Schema"}),`
`,e(r.p,{children:"This is probably the method you'll benefit from the most."}),`
`,e(r.p,{get children(){return["When schemas automatically retrieved from the ",e(r.a,{href:"https://www.schemastore.org",children:"JSON Schema Store"})," contain the following keys, automatic sorting is performed according to that schema:"]}}),`
`,e(r.ul,{get children(){return[`
`,e(r.li,{get children(){return e(r.a,{href:"/tombi/main/docs/json-schema#x-tombi-table-keys-order",children:"x-tombi-table-keys-order"})}}),`
`,e(r.li,{get children(){return e(r.a,{href:"/tombi/main/docs/json-schema#x-tombi-array-values-order",children:"x-tombi-array-values-order"})}}),`
`,e(r.li,{get children(){return e(r.a,{href:"/tombi/main/docs/json-schema#x-tombi-array-values-order-by",children:"x-tombi-array-values-order-by"})}}),`
`]}}),`
`,e(r.p,{get children(){return["Files like ",e(r.code,{children:"Cargo.toml"})," and ",e(r.code,{children:"pyproject.toml"})," registered in the JSON Schema Store have these keys specified."]}}),`
`,e(r.p,{get children(){return["Additionally, you can specify JSON Schema files using ",e(r.a,{href:"/tombi/main/docs/configuration",children:"configuration files"})," or ",e(r.a,{href:"/tombi/main/docs/comment-directive/tombi-value-directive#format-rules",children:"comment directives"}),", allowing you to apply schemas from sources other than the JSON Schema Store."]}}),`
`,e(r.h2,{id:"how-to-stop-auto-sorting",className:"group relative heading-with-anchor",children:"How to Stop Auto Sorting"}),`
`,e(r.p,{get children(){return["Auto sorting is defined for each JSON Schema file, and can be disabled for all schemas at once or for each schema in the ",e(r.a,{href:"/tombi/main/docs/configuration",children:"configuration file"}),"."]}}),`
`,e(r.h3,{id:"disable-auto-sorting-for-all-schemas",className:"group relative heading-with-anchor",children:"Disable Auto Sorting for All Schemas"}),`
`,e(r.p,{get children(){return["To disable schema-based auto sorting regardless of which schema applies, use ",e(r.a,{href:"/tombi/main/docs/configuration#schema-format-rules",get children(){return e(r.code,{children:"schema.format.rules"})}}),"."]}}),`
`,e(t,{code:`[schema.format.rules]
array-values-order.enabled = false
table-keys-order.enabled = false`,language:"toml"}),`
`,e(r.p,{get children(){return["This global setting is a fallback with lower priority than the same setting in ",e(r.a,{href:"/tombi/main/docs/configuration#schemas-format-rules",get children(){return e(r.code,{children:"schemas[*].format.rules"})}}),`.
For example, you can disable sorting globally and re-enable it for a specific schema.`]}}),`
`,e(t,{code:`[schema.format.rules]
array-values-order.enabled = false
table-keys-order.enabled = false

[[schemas]]
path = "tombi://www.schemastore.org/cargo.json"
include = ["Cargo.toml"]
format.rules.table-keys-order.enabled = true`,language:"toml"}),`
`,e(r.p,{get children(){return[e(r.a,{href:"/tombi/main/docs/formatter/auto-sorting#comment-directives",children:"Comment directives"})," and ",e(r.a,{href:"/tombi/main/docs/configuration#schemas-overrides",get children(){return e(r.code,{children:"schemas[*].overrides[*]"})}})," are still applied on top of this setting."]}}),`
`,e(r.h3,{id:"disable-auto-sorting-for-each-schema",className:"group relative heading-with-anchor",children:"Disable Auto Sorting for Each Schema"}),`
`,e(r.p,{get children(){return["If you want to disable automatic sorting for typical ",e(r.code,{children:"Cargo.toml"})," and ",e(r.code,{children:"pyproject.toml"}),` files,
please create the following configuration file.`]}}),`
`,e(t,{code:`[[schemas]]
path = "tombi://www.schemastore.org/cargo.json"
include = ["Cargo.toml"]
format.rules.array-values-order.enabled = false
format.rules.table-keys-order.enabled = false

[[schemas]]
path = "tombi://www.schemastore.org/pyproject.json"
include = ["pyproject.toml"]
format.rules.array-values-order.enabled = false
format.rules.table-keys-order.enabled = false`,language:"toml"}),`
`,e(r.p,{get children(){return[`If you want to keep automatic sorting only for specific keys after disabling it for a schema,
use `,e(r.a,{href:"/tombi/main/docs/configuration#schemas-overrides",get children(){return e(r.code,{children:"schemas[*].overrides[*]"})}})," to re-enable sorting for those targets."]}}),`
`,e(r.p,{get children(){return["For example, the following configuration disables sorting for ",e(r.code,{children:"Cargo.toml"}),` as a whole,
but keeps sorting only under `,e(r.code,{children:"[dependencies]"}),", ",e(r.code,{children:"[dev-dependencies]"}),", and ",e(r.code,{children:"[build-dependencies]"}),"."]}}),`
`,e(t,{code:`[[schemas]]
path = "tombi://www.schemastore.org/cargo.json"
include = ["Cargo.toml"]
format.rules.array-values-order.enabled = false
format.rules.table-keys-order.enabled = false

[[schemas.overrides]]
targets = ["dependencies", "dev-dependencies", "build-dependencies"]
format.rules = {
  table-keys-order = "ascending"
}`,language:"toml"})]}function u(o={}){const{wrapper:r}={...s(),...o.components};return r?e(r,d(o,{get children(){return e(i,o)}})):i(o)}function a(o,r){throw new Error("Expected component `"+o+"` to be defined: you likely forgot to import, pass, or provide it.")}export{u as default};
