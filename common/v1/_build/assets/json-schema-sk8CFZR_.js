import{b as e,k as a,i as s,t as g}from"./web-DCz9_uPG.js";import{M as o}from"./index-6OQr06v0.js";import{N as n,W as d}from"./Highlight-CMhf5ZLw.js";var u=g("<img src=/tombi/common/v1/x-tombi-additional-key-label.png class=w-full>");function h(i){const r={a:"a",br:"br",code:"code",h1:"h1",h2:"h2",h3:"h3",h4:"h4",li:"li",ol:"ol",p:"p",strong:"strong",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",ul:"ul",...o(),...i.components},{CodeBlock:t,PageHeading:c}=r;return t||l("CodeBlock"),c||l("PageHeading"),[e(c,{title:"JSON Schema",description:"Schema validation and completion with SchemaStore.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/json-schema"}),`
`,`
`,e(r.h1,{id:"json-schema",className:"group relative heading-with-anchor",children:"JSON Schema"}),`
`,e(r.p,{get children(){return["While TOML itself may introduce ",e(r.a,{href:"https://github.com/toml-lang/toml/issues/792",children:"schema specifications"}),` in the future,
Tombi, like Taplo, is implementing validation features in its linter that support `,e(r.a,{href:"https://json-schema.org/",children:"JSON Schema"}),"."]}}),`
`,e(r.p,{get children(){return["Currently, we are extending JSON Schema with special annotations using the ",e(r.code,{children:"x-tombi-*"})," prefix."]}}),`
`,e(r.h2,{id:"schema-priority",className:"group relative heading-with-anchor",children:"Schema Priority"}),`
`,e(r.p,{get children(){return["This section explains how Tombi prioritizes the application of ",e(r.code,{children:"x-tombi-*"})," keys in your JSON Schema:"]}}),`
`,e(r.ol,{get children(){return[`
`,e(r.li,{get children(){return[e(r.code,{children:"#:schema"})," directive in the TOML file's top comment (compatible with ",e(r.a,{href:"https://taplo.tamasfe.dev/configuration/directives.html#the-schema-directive",children:"Taplo"}),")"]}}),`
`,e(r.li,{get children(){return["JSON Schema specified in ",e(r.a,{href:"/tombi/common/v1/docs/configuration#search-priority",children:"the Tombi configuration file"})]}}),`
`,e(r.li,{get children(){return["JSON Schema from the ",e(r.a,{href:"https://www.schemastore.org",children:"JSON Schema Store"})]}}),`
`]}}),`
`,e(r.h2,{id:"private-git-hub-schemas",className:"group relative heading-with-anchor",children:"Private GitHub schemas"}),`
`,e(r.p,{get children(){return["The native CLI and editor language server can fetch schemas from private GitHub repositories using raw URLs. For example, add this directive to ",e(r.code,{children:"nagi.toml"}),":"]}}),`
`,e(t,{code:"#:schema https://raw.githubusercontent.com/nagi-sql/nagi-config/refs/heads/main/jsonschema/generated/nagi.json",language:"toml"}),`
`,e(r.p,{get children(){return["Or add a schema entry to ",e(r.code,{children:"tombi.toml"}),":"]}}),`
`,e(t,{code:`[[schemas]]
path = "https://raw.githubusercontent.com/nagi-sql/nagi-config/refs/heads/main/jsonschema/generated/nagi.json"
include = ["nagi.toml"]`,language:"toml"}),`
`,e(r.p,{get children(){return["Use a raw URL, rather than a GitHub ",e(r.code,{children:"blob"})," page URL. Tombi first fetches the schema without authentication. If GitHub returns HTTP 401, 403, or 404, Tombi retries with credentials in this order:"]}}),`
`,e(r.ol,{get children(){return[`
`,e(r.li,{get children(){return e(r.code,{children:"GH_TOKEN"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"GITHUB_TOKEN"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"gh auth token --hostname github.com"})}}),`
`]}}),`
`,e(r.p,{get children(){return["The account or token must have read access to the repository. If an environment token is set but invalid, Tombi reports the fetch failure without switching to another credential source. If neither token is set and ",e(r.code,{children:"gh"})," is unavailable or not signed in, fetching fails without opening a login prompt."]}}),`
`,e(r.p,{get children(){return["Credentials must be available to the Tombi process. For an editor running in a container, remote host, or WSL, configure the environment or ",e(r.code,{children:"gh"})," authentication there. This feature is not available in the browser/WASM build."]}}),`
`,e(n,{get children(){return e(r.p,{get children(){return["Tokens are used only for the authenticated request and are not cached. The downloaded schema JSON is cached locally, including schemas from private repositories. The default cache lifetime is 24 hours; offline mode can continue using the local copy. See ",e(r.a,{href:"/tombi/common/v1/docs/environment-variables",children:"environment variables"})," for cache controls."]}})}}),`
`,e(r.h2,{id:"built-in-schemas",className:"group relative heading-with-anchor",children:"Built-in schemas"}),`
`,e(r.p,{get children(){return["Tombi embeds a subset of JSON Schema Store schemas into the binary. To use an embedded schema, replace the ",e(r.code,{children:"https://"})," scheme with ",e(r.code,{children:"tombi://"}),"."]}}),`
`,e(r.p,{get children(){return["For example, use ",e(r.code,{children:"tombi://www.schemastore.org/cargo.json"})," instead of ",e(r.code,{children:"https://www.schemastore.org/cargo.json"}),"."]}}),`
`,e(r.p,{children:"The authoritative list of embedded schemas is available from the embedded catalog:"}),`
`,e(r.ul,{get children(){return[`
`,e(r.li,{get children(){return e(r.code,{children:"tombi://www.schemastore.org/api/json/catalog.json"})}}),`
`]}}),`
`,e(r.p,{children:"Some common embedded schema URIs include:"}),`
`,e(r.ul,{get children(){return[`
`,e(r.li,{get children(){return e(r.code,{children:"tombi://www.schemastore.org/cargo.json"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"tombi://www.schemastore.org/pyproject.json"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"tombi://www.schemastore.org/tombi.json"})}}),`
`]}}),`
`,e(n,{get children(){return e(r.p,{children:`Built-in schemas are useful when you want schema validation and completion to work offline, or when you want to avoid fetching schemas over the network.
The embedded catalog above is the source of truth for which schemas are built into the current binary.`})}}),`
`,e(r.h2,{id:"tombi-metadata",className:"group relative heading-with-anchor",children:"Tombi metadata"}),`
`,e(r.p,{children:"Tombi supports the following metadata keys in your JSON Schema."}),`
`,e(r.h3,{id:"x-tombi-toml-version",className:"group relative heading-with-anchor",children:"x-tombi-toml-version"}),`
`,e(r.p,{children:`This key automatically determines the TOML version to use.
Currently, we support:`}),`
`,e(r.ul,{get children(){return[`
`,e(r.li,{get children(){return[e(r.code,{children:"v1.0.0"})," (stable)"]}}),`
`,e(r.li,{get children(){return[e(r.code,{children:"v1.1.0"})," (latest)"]}}),`
`]}}),`
`,e(r.p,{children:"The preview version includes exciting features like trailing comma support in Inline Tables."}),`
`,e(n,{get children(){return[e(r.p,{get children(){return[e(r.code,{children:"v1.1.0"}),` has only just been released and is not yet supported by many tools.
Therefore, `,e(r.code,{children:"v1.0.0"})," will be used as the default TOML version for the time being."]}}),e(r.p,{get children(){return["To override ",e(r.code,{children:"x-tombi-toml-version"})," with an older version, set ",e(r.code,{children:"toml-version"})," in each relevant ",e(r.code,{children:"[[schemas]]"})," entry in your ",e(r.code,{children:"tombi.toml"}),"."]}}),e(t,{code:`[[schemas]]
toml-version = "v1.0.0"
path = "tombi://www.schemastore.org/cargo.json"
include = ["Cargo.toml"]`,language:"toml"})]}}),`
`,e(r.h3,{id:"x-tombi-additional-key-label",className:"group relative heading-with-anchor",children:"x-tombi-additional-key-label"}),`
`,e(r.p,{children:"This key specifies the label to use for additional keys."}),`
`,e(t,{code:`{
  "type": "object",
  "properties": {
    "dependencies": {
      "type": "object",
      "additionalProperties": {
        "$ref": "#/definitions/Dependency"
      },
      "x-tombi-additional-key-label": "crate_name",
      ...
    },
    ...
  },
  ...
}`,language:"json"}),`
`,e(r.p,{get children(){return["This allows the label displayed in the completion candidates for additionalProperties keys to be ",e(r.code,{children:"$crate_name"}),".",e(r.br,{}),`
`,"If you don't specify this key, the label will be ",e(r.code,{children:"$key"}),"."]}}),`
`,s(u),`
`,e(r.h2,{id:"formatting",className:"group relative heading-with-anchor",children:"Formatting"}),`
`,e(r.h3,{id:"x-tombi-table-keys-order",className:"group relative heading-with-anchor",children:"x-tombi-table-keys-order"}),`
`,e(r.p,{get children(){return["This key controls the automatic sorting of table keys (e.g., ",e(r.code,{children:"[dependencies]"}),")."]}}),`
`,e(r.p,{children:"Available sorting strategies:"}),`
`,e(r.ul,{get children(){return[`
`,e(r.li,{get children(){return e(r.code,{children:"ascending"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"descending"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"version-sort"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"schema"})}}),`
`]}}),`
`,e(n,{get children(){return e(r.p,{get children(){return["The ",e(r.code,{children:"version-sort"})," strategy is based on the ",e(r.a,{href:"https://doc.rust-lang.org/nightly/style-guide/#sorting",children:"Rust style guide"}),"."]}})}}),`
`,e(r.p,{get children(){return["By specifying object instead of string, you can individually define sorting methods for the different groups: ",e(r.code,{children:"properties"}),", ",e(r.code,{children:"additionalProperties"}),", and ",e(r.code,{children:"patternProperties"}),"."]}}),`
`,e(t,{code:`{
  "x-tombi-table-keys-order": {
    "properties": "schema",
    "patternProperties": "version-sort",
    "additionalProperties": "descending"
  }
}`,language:"json"}),`
`,e(r.p,{children:"The order of the object's properties is significant. Sorting is performed sequentially for each group starting from the first property, and the results are then combined."}),`
`,e(n,{get children(){return e(r.p,{get children(){return["The ",e(r.code,{children:"schema"})," strategy can only be used for ",e(r.code,{children:"properties"}),"."]}})}}),`
`,e(r.h3,{id:"x-tombi-array-values-order",className:"group relative heading-with-anchor",children:"x-tombi-array-values-order"}),`
`,e(r.p,{children:"This key controls the automatic sorting of array values."}),`
`,e(r.p,{children:"Available sorting strategies:"}),`
`,e(r.ul,{get children(){return[`
`,e(r.li,{get children(){return e(r.code,{children:"ascending"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"descending"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"version-sort"})}}),`
`]}}),`
`,e(n,{get children(){return e(r.p,{get children(){return["The ",e(r.code,{children:"version-sort"})," strategy is based on the ",e(r.a,{href:"https://doc.rust-lang.org/nightly/style-guide/#sorting",children:"Rust style guide"}),"."]}})}}),`
`,e(d,{get children(){return e(r.p,{get children(){return[e(r.code,{children:"Float"})," and ",e(r.code,{children:"Array"})," cannot be sorted. To sort ",e(r.code,{children:"Table"}),", you need to use ",e(r.code,{children:"x-tombi-array-values-order-by"}),"."]}})}}),`
`,e(r.p,{get children(){return["By specifying object instead of string, you can individually define sorting methods for the different groups: ",e(r.code,{children:"oneOf"}),", ",e(r.code,{children:"anyOf"}),"."]}}),`
`,e(r.p,{get children(){return["The following example, sorts the first schema of oneOf with ",e(r.code,{children:"version-sort"}),", and the second schema with ",e(r.code,{children:"ascending"})," after that."]}}),`
`,e(t,{code:`{
  "type": "array",
  "items": {
    "oneOf": [
      { "type": "string" },
      { "type": "number" }
    ]
  },
  "x-tombi-array-values-order": {
    "oneOf": [
      "version-sort",
      "ascending"
    ]
  }
}`,language:"json"}),`
`,e(r.p,{children:"The following is the sorting result."}),`
`,e(t,{code:`# Before
key = [1, "2", 3, "4"]

# After
key = ["2", "4", 1, 3]`,language:"toml"}),`
`,e(d,{get children(){return[e(r.p,{get children(){return[e(r.code,{children:"oneOf"})," and ",e(r.code,{children:"anyOf"})," cannot sort nested schemas. ",e(r.code,{children:"oneOf"})," and ",e(r.code,{children:"anyOf"})," must be flattened."]}}),e(t,{code:`{
  "oneOf": [
    {
      // Cannot use nested \`oneOf\` / \`anyOf\`
      "oneOf": [
        { "type": "string" },
        { "type": "number" }
      ]
    },
    ...
  ],
  "x-tombi-array-values-order": {
    "oneOf": [...]
  }
}`,language:"json"})]}}),`
`,e(r.h3,{id:"x-tombi-array-values-order-by",className:"group relative heading-with-anchor",children:"x-tombi-array-values-order-by"}),`
`,e(r.p,{children:"This key specifies which key to use as the sorting key when the array element is a table."}),`
`,e(t,{code:`{
  "type": "array",
  "items": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string"
      },
      "age": {
        "type": "number"
      }
    },
    "x-tombi-array-values-order-by": "name"
  },
  "x-tombi-array-values-order": "ascending"
}`,language:"json"}),`
`,e(r.p,{children:"The following is the sorting result."}),`
`,e(t,{code:`# Before
key = [
  { name = "Bob", age = 30 },
  { name = "Charlie", age = 25 },
  { name = "Alice", age = 20 },
]

# After
key = [
  { name = "Alice", age = 20 },
  { name = "Bob", age = 30 },
  { name = "Charlie", age = 25 },
]`,language:"toml"}),`
`,e(r.h2,{id:"linting",className:"group relative heading-with-anchor",children:"Linting"}),`
`,e(r.h3,{id:"validation-score",className:"group relative heading-with-anchor",children:"Validation Score"}),`
`,e(r.p,{children:`For complex JSON Schemas, validation failures can produce many error messages.
To help users identify the most relevant errors,
Tombi uses a scoring system to filter out less important validation failures.`}),`
`,e(r.p,{children:"The currently implemented points are as follows:"}),`
`,e(r.ul,{get children(){return[`
`,e(r.li,{children:"Type matching: 1 point is added for the value that matches the type defined in the JSON Schema."}),`
`,e(r.li,{children:"Required key matching: 1 point is added for each required key present in the table schema."}),`
`]}}),`
`,e(r.p,{get children(){return["More details are ",e(r.a,{href:"https://github.com/tombi-toml/tombi/issues/1021#issuecomment-3338983593",children:"here"}),"."]}}),`
`,e(r.h3,{id:"strict-mode",className:"group relative heading-with-anchor",children:"Strict Mode"}),`
`,e(r.p,{get children(){return["By default, Tombi operates in ",e(r.code,{children:"strict"})," mode. In this mode, objects without ",e(r.code,{children:"additionalProperties"})," are treated as if ",e(r.code,{children:"additionalProperties: false"}),` was specified.
This differs from the standard JSON Schema specification but provides more precise validation by eliminating ambiguity.`]}}),`
`,e(r.p,{get children(){return["To disable strict mode globally, add ",e(r.code,{children:"schema.strict = false"})," to your ",e(r.code,{children:"tombi.toml"}),` configuration.
You can also set `,e(r.code,{children:"strict"})," on a matching ",e(r.code,{children:"[[schemas]]"})," root or sub-schema entry."]}}),`
`,e(r.p,{children:"Tombi resolves strict mode in this order:"}),`
`,e(r.ol,{get children(){return[`
`,e(r.li,{get children(){return["The document comment directive: ",e(r.code,{children:"#:tombi schema.strict = true"})," or ",e(r.code,{children:"false"})]}}),`
`,e(r.li,{get children(){return["The matching sub-schema ",e(r.code,{children:"strict"})," setting"]}}),`
`,e(r.li,{get children(){return["The matching root-schema ",e(r.code,{children:"strict"})," setting"]}}),`
`,e(r.li,{get children(){return["The global ",e(r.code,{children:"schema.strict"})," setting"]}}),`
`,e(r.li,{get children(){return["The built-in default, ",e(r.code,{children:"true"})]}}),`
`]}}),`
`,e(r.p,{get children(){return["An omitted level falls through to the next one. A sub-schema setting applies only at and below its configured ",e(r.code,{children:"root"}),"."]}}),`
`,e(r.h3,{id:"x-tombi-string-formats",className:"group relative heading-with-anchor",children:"x-tombi-string-formats"}),`
`,e(r.p,{get children(){return["In Tombi default behavior, the ",e(r.code,{children:"format"})," used in JSON Schema's ",e(r.code,{children:'“type”: “string"'})," only recognizes conversions to the following built-in types:"]}}),`
`,e(r.ul,{get children(){return[`
`,e(r.li,{get children(){return e(r.code,{children:"date-time"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"date-time-local"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"date"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"time-local"})}}),`
`]}}),`
`,e(r.p,{get children(){return["By adding the ",e(r.code,{children:"x-tombi-string-formats"})," key to the root of the JSON Schema, you can add the ",e(r.code,{children:"format"})," of the string to the validation target."]}}),`
`,e(t,{code:`{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "x-tombi-string-formats": [
    "email",
    "hostname",
    "uri",
    "uuid",
    "ipv4",
    "ipv6"
  ]
}`,language:"json"}),`
`,e(r.p,{get children(){return["Currently supported ",e(r.code,{children:"format"})," targets are as follows:"]}}),`
`,e(r.table,{get children(){return[e(r.thead,{get children(){return e(r.tr,{get children(){return[e(r.th,{children:"format"}),e(r.th,{children:"Specification"}),e(r.th,{children:"TOML Type"})]}})}}),e(r.tbody,{get children(){return[e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"email"})}}),e(r.td,{get children(){return e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc5322",children:"RFC 5322"})}}),e(r.td,{})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"hostname"})}}),e(r.td,{get children(){return e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc1034",children:"RFC 1034"})}}),e(r.td,{})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"uri"})}}),e(r.td,{get children(){return e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc3986",children:"RFC 3986"})}}),e(r.td,{})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"uri-reference"})}}),e(r.td,{get children(){return e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc3986",children:"RFC 3986"})}}),e(r.td,{})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"uuid"})}}),e(r.td,{get children(){return e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc4122",children:"RFC 4122"})}}),e(r.td,{})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"ipv4"})}}),e(r.td,{get children(){return e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc2673#section-3.2",children:"RFC 2673 §3.2"})}}),e(r.td,{})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"ipv6"})}}),e(r.td,{get children(){return e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc4291#section-2.2",children:"RFC 4291 §2.2"})}}),e(r.td,{})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"date-time"})}}),e(r.td,{get children(){return e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc3339#section-5.6",children:"RFC 3339 §5.6"})}}),e(r.td,{children:"Offset Date-Time"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"date-time-local"})}}),e(r.td,{get children(){return e(r.a,{href:"https://spec.openapis.org/registry/format/date-time-local.html",children:"OpenAPI Format Registry"})}}),e(r.td,{children:"Local Date-Time"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"date"})}}),e(r.td,{get children(){return[e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc3339#section-5.6",children:"RFC 3339 §5.6"})," full-date"]}}),e(r.td,{children:"Local Date"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"time"})}}),e(r.td,{get children(){return[e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc3339#section-5.6",children:"RFC 3339 §5.6"})," full-time"]}}),e(r.td,{})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"time-local"})}}),e(r.td,{get children(){return e(r.a,{href:"https://spec.openapis.org/registry/format/time-local.html",children:"OpenAPI Format Registry"})}}),e(r.td,{children:"Local Time"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"regex"})}}),e(r.td,{get children(){return e(r.a,{href:"https://262.ecma-international.org/",children:"ECMA-262"})}}),e(r.td,{})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"json-pointer"})}}),e(r.td,{get children(){return e(r.a,{href:"https://datatracker.ietf.org/doc/html/rfc6901",children:"RFC 6901"})}}),e(r.td,{})]}})]}})]}}),`
`,e(n,{get children(){return e(r.p,{get children(){return["If you want to support additional ",e(r.code,{children:"format"})," in Tombi, please check if it exists in ",e(r.a,{href:"https://json-schema.org/draft/2020-12/json-schema-validation#name-defined-formats",children:"JSON Schema Specification"})," or ",e(r.a,{href:"https://spec.openapis.org/registry/format/",children:"OpenAPI Format Registry"}),"."]}})}}),`
`,e(r.h2,{id:"language-gap",className:"group relative heading-with-anchor",children:"Language Gap"}),`
`,e(r.p,{children:"While TOML and JSON are different languages, JSON Schema remains a valuable tool for representing TOML structures, especially given the abundance of existing schema assets."}),`
`,e(r.p,{children:"Tombi bridges this language gap by using abbreviations to represent JSON Schema concepts that don't have direct equivalents in TOML."}),`
`,e(r.p,{children:"For more details on how Tombi represents these concepts, check out the Hover section."}),`
`,e(r.h2,{id:"json-catalog-schema-cache",className:"group relative heading-with-anchor",children:"JSON Catalog/Schema Cache"}),`
`,e(r.p,{children:"Tombi caches JSON Catalog/Schemas in the file system to avoid unnecessary network requests."}),`
`,e(r.h3,{id:"cache-expiration",className:"group relative heading-with-anchor",children:"Cache Expiration"}),`
`,e(r.p,{get children(){return["Cached schemas expire after ",e(r.strong,{children:"24 hours"})," by default. When a cached schema expires, Tombi automatically fetches the latest version from the remote source."]}}),`
`,e(r.p,{get children(){return["Use ",e(r.code,{children:"--cache-ttl <SECONDS>"})," to set the cache expiration time for ",e(r.code,{children:"tombi format"}),", ",e(r.code,{children:"tombi lint"}),", or ",e(r.code,{children:"tombi lsp"}),":"]}}),`
`,e(t,{code:"tombi format --cache-ttl 3600",language:"sh"}),`
`,e(r.p,{get children(){return["The value must be a non-negative integer in seconds. The CLI option overrides the ",e(r.code,{children:"TOMBI_CACHE_TTL"})," environment variable. When neither is set, the default is ",e(r.code,{children:"86400"})," seconds (24 hours). See ",e(r.a,{href:"/tombi/common/v1/docs/environment-variables#tombi-cache-ttl",children:"Environment Variables"})," for details."]}}),`
`,e(r.h3,{id:"how-to-clear-cache",className:"group relative heading-with-anchor",children:"How to Clear Cache"}),`
`,e(r.h4,{id:"cli",className:"group relative heading-with-anchor",children:"CLI"}),`
`,e(r.p,{get children(){return["Run ",e(r.code,{children:"tombi cache clean"})," to remove all cached catalogs and schemas. The cache directory remains in place. Use ",e(r.code,{children:"tombi cache dir"})," to print its absolute path, including when the directory does not exist yet."]}}),`
`,e(r.p,{get children(){return["When ",e(r.code,{children:"TOMBI_CACHE_HOME"})," is set, cleaning removes every entry in that directory. Use ",e(r.code,{children:"tombi cache dir"})," to check the selected path. Avoid a shared directory that another process can rename or replace while cleaning runs."]}}),`
`,e(t,{code:`tombi cache dir
tombi cache clean`,language:"bash"}),`
`,e(r.p,{get children(){return["Use ",e(r.code,{children:"--no-cache"})," option to disable using caches for a CLI command."]}}),`
`,e(t,{code:"tombi format --no-cache",language:"bash"}),`
`,e(d,{get children(){return[e(r.p,{get children(){return[e(r.code,{children:"--no-cache"})," option does not use caches, but ",e(r.strong,{children:"saves the latest catalogs/schemas"})," to the file caches.",e(r.br,{}),`
`,"(This behavior is the same as HTTP's ",e(r.code,{children:"Cache-Control: no-cache"}),")"]}}),e(r.p,{children:"It only applies to catalogs/schemas used by the command, and does not update other files."})]}}),`
`,e(r.h4,{id:"language-server",className:"group relative heading-with-anchor",children:"Language Server"}),`
`,e(r.p,{get children(){return["Use ",e(r.code,{children:"RefreshCache"})," command to clear the cache."]}}),`
`,e(d,{get children(){return e(r.p,{get children(){return["It removes ",e(r.strong,{children:"all file caches"})," and fetches the latest schema over the network for the editing TOML file."]}})}}),`
`,e(r.h3,{id:"cache-location-search-priority",className:"group relative heading-with-anchor",children:"Cache Location Search Priority"}),`
`,e(r.ol,{get children(){return[`
`,e(r.li,{get children(){return e(r.code,{children:"$TOMBI_CACHE_HOME"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"$XDG_CACHE_HOME/tombi"})}}),`
`,e(r.li,{get children(){return e(r.code,{children:"~/.cache/tombi"})}}),`
`]}}),`
`,e(r.h2,{id:"associate-schema",className:"group relative heading-with-anchor",children:"Associate Schema"}),`
`,e(r.p,{get children(){return[`VSCode supports associating a TOML schema with a file match pattern.
See `,e(r.a,{href:"/tombi/common/v1/docs/editors/vscode-extension#json-schema-association",children:"VSCode Extension"})," for details."]}}),`
`,e(r.h2,{id:"non-standard-supported-features",className:"group relative heading-with-anchor",children:"Non-standard Supported Features"}),`
`,e(r.p,{children:`Tombi supports the following features for compatibility with existing schema tooling, even though
they are not part of the JSON Schema specification.`}),`
`,e(r.h3,{id:"deprecation-message",className:"group relative heading-with-anchor",children:"deprecationMessage"}),`
`,e(r.p,{get children(){return[e(r.code,{children:"deprecationMessage"}),` is a non-standard keyword originating from the VS Code JSON language service.
Tombi accepts it for compatibility, but only honors it alongside `,e(r.code,{children:"deprecated: true"}),"."]}}),`
`,e(r.p,{get children(){return["When both are present, the ",e(r.code,{children:"deprecationMessage"}),` string is used as the deprecation diagnostic message
instead of the generic one. A `,e(r.code,{children:"deprecationMessage"})," without ",e(r.code,{children:"deprecated: true"})," is ignored."]}}),`
`,e(t,{code:`{
  "properties": {
    "old-key": {
      "type": "string",
      "deprecated": true,
      "deprecationMessage": "Use \`new-key\` instead."
    }
  }
}`,language:"json"}),`
`,e(r.h2,{id:"compliance-status",className:"group relative heading-with-anchor",children:"Compliance Status"}),`
`,e(r.p,{children:`The table below summarizes keyword membership in each JSON Schema dialect and whether Tombi
implements the keyword when that dialect defines it.`}),`
`,e(r.p,{get children(){return[e(r.code,{children:"✅"}),": supported in the dialect",e(r.br,{}),`
`,e(r.code,{children:"❌"}),": not implemented in Tombi",e(r.br,{}),`
`,e(r.code,{children:"-"}),": not used by that dialect"]}}),`
`,e(r.table,{get children(){return[e(r.thead,{get children(){return e(r.tr,{get children(){return[e(r.th,{children:"Keyword"}),e(r.th,{children:"draft-07"}),e(r.th,{children:"draft-2019-09"}),e(r.th,{children:"draft-2020-12"})]}})}}),e(r.tbody,{get children(){return[e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$id"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$schema"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$ref"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$defs"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"definitions"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"-"}),e(r.td,{children:"-"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$anchor"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$dynamicRef"})}}),e(r.td,{children:"-"}),e(r.td,{children:"-"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$dynamicAnchor"})}}),e(r.td,{children:"-"}),e(r.td,{children:"-"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$recursiveRef"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"-"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$recursiveAnchor"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"-"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$vocabulary"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"allOf"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"anyOf"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"oneOf"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"not"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"if"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"then"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"else"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"properties"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"patternProperties"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"additionalProperties"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"propertyNames"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"items"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"additionalItems"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"-"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"prefixItems"})}}),e(r.td,{children:"-"}),e(r.td,{children:"-"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"contains"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"dependentSchemas"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"unevaluatedProperties"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"unevaluatedItems"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"type"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"enum"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"const"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"multipleOf"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"maximum"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"minimum"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"exclusiveMaximum"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"exclusiveMinimum"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"maxLength"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"minLength"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"pattern"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"format"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"maxItems"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"minItems"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"uniqueItems"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"maxProperties"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"minProperties"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"required"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"dependencies"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"-"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"dependentRequired"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"minContains"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"maxContains"})}}),e(r.td,{children:"-"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"title"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"description"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"default"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"examples"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"deprecated"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"$comment"})}}),e(r.td,{children:"❌"}),e(r.td,{children:"❌"}),e(r.td,{children:"❌"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"readOnly"})}}),e(r.td,{children:"❌"}),e(r.td,{children:"❌"}),e(r.td,{children:"❌"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"writeOnly"})}}),e(r.td,{children:"❌"}),e(r.td,{children:"❌"}),e(r.td,{children:"❌"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"contentEncoding"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"contentMediaType"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}}),e(r.tr,{get children(){return[e(r.td,{get children(){return e(r.code,{children:"contentSchema"})}}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"}),e(r.td,{children:"✅"})]}})]}})]}}),`
`,e(n,{get children(){return e(r.p,{get children(){return[`Tombi also accepts some legacy keywords for compatibility even when they are not part of the
selected dialect. For example, `,e(r.code,{children:"definitions"})," is still loaded alongside ",e(r.code,{children:"$defs"}),` in draft-2019-09
and draft-2020-12, `,e(r.code,{children:"dependencies"})," is still honored in draft-2020-12, and ",e(r.code,{children:"prefixItems"}),` is
accepted even before draft-2020-12.`]}})}}),`
`,e(n,{get children(){return e(r.p,{get children(){return[e(r.code,{children:"format"}),", ",e(r.code,{children:"title"}),", ",e(r.code,{children:"description"}),", ",e(r.code,{children:"default"}),", ",e(r.code,{children:"examples"}),", ",e(r.code,{children:"deprecated"}),", ",e(r.code,{children:"contentEncoding"}),", ",e(r.code,{children:"contentMediaType"}),", and ",e(r.code,{children:"contentSchema"}),` are not all treated as pure assertion keywords.
Some are used as annotations for validation behavior, hover, completion, or diagnostics.`]}})}}),`
`,e(n,{get children(){return e(r.p,{get children(){return["Tombi defaults to a non-standard ",e(r.code,{children:"strict"})," behavior: when ",e(r.code,{children:"additionalProperties"})," is omitted, objects are treated as closed unless you disable ",e(r.code,{children:"schema.strict"}),"."]}})}})]}function b(i={}){const{wrapper:r}={...o(),...i.components};return r?e(r,a(i,{get children(){return e(h,i)}})):h(i)}function l(i,r){throw new Error("Expected component `"+i+"` to be defined: you likely forgot to import, pass, or provide it.")}export{b as default};
