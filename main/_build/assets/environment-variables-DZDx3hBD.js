import{M as o}from"./index-6OQr06v0.js";import{b as n,k as s}from"./web-DCz9_uPG.js";function i(t){const e={code:"code",h1:"h1",h2:"h2",li:"li",p:"p",ul:"ul",...o(),...t.components},{PageHeading:r}=e;return r||a("PageHeading"),[n(r,{title:"Environment Variables",description:"Configure Tombi using environment variables.",og_url:"https://tombi-toml.github.io/tombi/main/docs/environment-variables"}),`
`,n(e.h1,{id:"environment-variables",className:"group relative heading-with-anchor",children:"Environment Variables"}),`
`,n(e.p,{children:"Tombi CLI supports several environment variables to configure its behavior."}),`
`,n(e.h2,{id:"tombi-cache-home",className:"group relative heading-with-anchor",children:"TOMBI_CACHE_HOME"}),`
`,n(e.p,{children:"Set the cache directory path for JSON Catalog/Schemas explicitly."}),`
`,n(e.p,{children:"When set, Tombi stores and reads cached schemas from this directory before considering any XDG or home-directory based fallback. This is useful in hermetic environments, such as Nix pipelines, where the cache location must be provided explicitly."}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{get children(){return["Type: ",n(e.code,{children:"Path"})]}}),`
`,n(e.li,{children:"Default: unset"}),`
`]}}),`
`,n(e.h2,{id:"tombi-offline",className:"group relative heading-with-anchor",children:"TOMBI_OFFLINE"}),`
`,n(e.p,{children:"Disable network access and use only local schema cache."}),`
`,n(e.p,{children:"When set, Tombi will not fetch schemas from remote sources and will only use cached schemas locally. This is useful when working in offline environments or when you want to ensure no network requests are made."}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{get children(){return["Type: ",n(e.code,{children:"true | false"})]}}),`
`,n(e.li,{get children(){return["Default: ",n(e.code,{children:"false"})]}}),`
`]}}),`
`,n(e.h2,{id:"tombi-no-color",className:"group relative heading-with-anchor",children:"TOMBI_NO_COLOR"}),`
`,n(e.p,{children:"Disable ANSI color output from Tombi CLI diagnostics."}),`
`,n(e.p,{get children(){return["When set to a non-empty value, Tombi will not emit ANSI color sequences in CLI diagnostics. If ",n(e.code,{children:"TOMBI_NO_COLOR"})," is not set, Tombi will fall back to ",n(e.code,{children:"NO_COLOR"})," if available."]}}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{get children(){return["Type: ",n(e.code,{children:"String"})]}}),`
`,n(e.li,{children:"Default: unset"}),`
`]}}),`
`,n(e.h2,{id:"tombi-no-cache",className:"group relative heading-with-anchor",children:"TOMBI_NO_CACHE"}),`
`,n(e.p,{children:"Disable cache usage and fetch the latest data from remote sources."}),`
`,n(e.p,{children:"When set, Tombi will bypass the cache and fetch the latest schemas from remote sources, then save them to the cache. This is useful when you want to ensure you have the most up-to-date schema data."}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{get children(){return["Type: ",n(e.code,{children:"true | false"})]}}),`
`,n(e.li,{get children(){return["Default: ",n(e.code,{children:"false"})]}}),`
`]}}),`
`,n(e.h2,{id:"tombi-cache-ttl",className:"group relative heading-with-anchor",children:"TOMBI_CACHE_TTL"}),`
`,n(e.p,{children:"Set the cache expiration time (Time To Live) in seconds for JSON Catalog/Schemas."}),`
`,n(e.p,{children:"When set, Tombi uses this value as the cache expiration time instead of the default 24 hours. Cached schemas older than this duration are refreshed from remote sources."}),`
`,n(e.p,{get children(){return["The ",n(e.code,{children:"--cache-ttl <SECONDS>"})," option for ",n(e.code,{children:"tombi format"}),", ",n(e.code,{children:"tombi lint"}),", and ",n(e.code,{children:"tombi lsp"})," overrides this environment variable. Values must be non-negative integers. The CLI rejects an invalid environment variable value."]}}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{get children(){return["Type: ",n(e.code,{children:"Integer"})," (seconds)"]}}),`
`,n(e.li,{get children(){return["Default: ",n(e.code,{children:"86400"})," (24 hours)"]}}),`
`]}}),`
`,n(e.h2,{id:"tombi-http-timeout",className:"group relative heading-with-anchor",children:"TOMBI_HTTP_TIMEOUT"}),`
`,n(e.p,{children:"Set the HTTP timeout in seconds for fetching remote schemas."}),`
`,n(e.p,{get children(){return["This environment variable controls how long Tombi will wait for HTTP requests to complete before timing out. If ",n(e.code,{children:"TOMBI_HTTP_TIMEOUT"})," is not set, Tombi will fall back to ",n(e.code,{children:"HTTP_TIMEOUT"})," if available."]}}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{get children(){return["Type: ",n(e.code,{children:"Integer"})," (seconds)"]}}),`
`,n(e.li,{get children(){return["Default: ",n(e.code,{children:"5"})]}}),`
`]}}),`
`,n(e.h2,{id:"tombi-schema-trusted-hosts",className:"group relative heading-with-anchor",children:"TOMBI_SCHEMA_TRUSTED_HOSTS"}),`
`,n(e.p,{children:"Allow schema and catalog requests to connect to non-public addresses returned for these exact host names or IP addresses."}),`
`,n(e.p,{get children(){return["Set a comma-separated list such as ",n(e.code,{children:"schema.internal,10.20.30.40"}),". Entries are exact hosts; wildcards are not supported. A trusted host grants access to its non-public addresses over any HTTP or HTTPS port, so add only hosts you trust. Public destinations remain available without this setting."]}}),`
`,n(e.p,{get children(){return["Native HTTP requests reject credentials in URLs, reject private or special-use destination addresses, and recheck every redirect. Proxies configured through ",n(e.code,{children:"HTTP_PROXY"}),", ",n(e.code,{children:"HTTPS_PROXY"}),", ",n(e.code,{children:"ALL_PROXY"})," and ",n(e.code,{children:"NO_PROXY"})," are honored, and the proxy host itself may have a private address. If these variables are not set, the operating-system proxy settings are used (macOS System Configuration, Windows registry); PAC files and automatic proxy discovery are not supported. When a request goes through a proxy, the proxy resolves the destination, so Tombi first resolves it locally and refuses it if it points to a non-public address; a destination that only the proxy can resolve, or that resolves differently for the proxy, cannot be checked, so the proxy is the egress control point. Schema URLs that point at the proxy host itself are refused. The browser/WASM build can reject unsafe URL literals and redirects, but the browser controls DNS and socket routing."]}}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{children:"Type: comma-separated host names or IP addresses"}),`
`,n(e.li,{children:"Default: unset"}),`
`]}})]}function h(t={}){const{wrapper:e}={...o(),...t.components};return e?n(e,s(t,{get children(){return n(i,t)}})):i(t)}function a(t,e){throw new Error("Expected component `"+t+"` to be defined: you likely forgot to import, pass, or provide it.")}export{h as default};
