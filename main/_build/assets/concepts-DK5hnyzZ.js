import{M as i}from"./index-6OQr06v0.js";import{b as o,k as a}from"./web-DCz9_uPG.js";function r(n){const e={a:"a",br:"br",code:"code",h1:"h1",h2:"h2",p:"p",...i(),...n.components},{PageHeading:t}=e;return t||c("PageHeading"),[o(t,{title:"Concepts",description:"Core concepts and features of Tombi.",og_url:"https://tombi-toml.github.io/tombi/main/docs/concepts"}),`
`,o(e.h1,{id:"concepts",className:"group relative heading-with-anchor",children:"Concepts"}),`
`,o(e.p,{children:"Tombi's concept is to allow any team or project to work on a TOML file with the same rules from the beginning."}),`
`,o(e.h2,{id:"zero-configuration",className:"group relative heading-with-anchor",children:"🚀 Zero Configuration"}),`
`,o(e.p,{children:`Tombi works out of the box with zero configuration.
If needed, you can still customize its behavior to match your team's preferences.`}),`
`,o(e.p,{get children(){return["See ",o(e.a,{href:"/tombi/main/docs/configuration",children:"Configuration"}),", ",o(e.a,{href:"/tombi/main/docs/comment-directive",children:"Comment Directive"}),", and ",o(e.a,{href:"/tombi/main/docs/json-schema",children:"JSON Schema"})," for more details."]}}),`
`,o(e.h2,{id:"schema-driven",className:"group relative heading-with-anchor",children:"🛤️ Schema Driven"}),`
`,o(e.p,{get children(){return["TOML is used in multiple programming languages and tools, so it needs to be formatted according to their rules.",o(e.br,{}),`
`,`Don't worry! You don't need to specify the rules,
Tombi will automatically apply the rules by getting the schema from the `,o(e.a,{href:"https://www.schemastore.org",children:"JSON Schema Store"}),"."]}}),`
`,o(e.p,{get children(){return["Those rules are determined by the schema provider, not your team.",o(e.br,{}),`
`,"Just like new programming languages come with formatters from the beginning, the schema provider determines those rules as well."]}}),`
`,o(e.h2,{id:"auto-sorting",className:"group relative heading-with-anchor",children:"↕️ Auto Sorting"}),`
`,o(e.p,{get children(){return["Tombi ",o(e.a,{href:"/tombi/main/docs/formatter/auto-sorting",children:"automatically sorts keys"})," in your TOML files based on the JSON Schema definition.",o(e.br,{}),`
`,"Using ",o(e.code,{children:"x-tombi-*"})," extension properties in JSON Schema, schema providers can define the optimal key order."]}}),`
`,o(e.p,{get children(){return["This ensures consistency across your configuration files without manual intervention.",o(e.br,{}),`
`,"Enjoy consistent formatting for your TOML files across all projects!"]}})]}function m(n={}){const{wrapper:e}={...i(),...n.components};return e?o(e,a(n,{get children(){return o(r,n)}})):r(n)}function c(n,e){throw new Error("Expected component `"+n+"` to be defined: you likely forgot to import, pass, or provide it.")}export{m as default};
