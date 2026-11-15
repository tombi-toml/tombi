import{M as c}from"./index-6OQr06v0.js";import{b as n,k as s}from"./web-DCz9_uPG.js";function i(o){const e={a:"a",code:"code",h1:"h1",h2:"h2",p:"p",...c(),...o.components},{CodeBlock:t,PageHeading:r}=e;return t||a("CodeBlock"),r||a("PageHeading"),[n(r,{title:"Formatter and Linter",description:"Use the tombi-wasm-lib package for formatting and linting.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/wasm/tombi-wasm-lib"}),`
`,n(e.h1,{id:"tombi-wasm-lib",className:"group relative heading-with-anchor",get children(){return n(e.code,{children:"tombi-wasm-lib"})}}),`
`,n(e.p,{get children(){return[n(e.code,{children:"tombi-wasm-lib"}),` provides Tombi's formatter and linter for browsers. For
server-side Node.js, use `,n(e.a,{href:"/tombi/common/v1/docs/library/nodejs",get children(){return n(e.code,{children:"tombi-lib"})}})," instead."]}}),`
`,n(e.h2,{id:"install",className:"group relative heading-with-anchor",children:"Install"}),`
`,n(t,{code:"pnpm add tombi-wasm-lib",language:"sh"}),`
`,n(e.h2,{id:"format-toml",className:"group relative heading-with-anchor",children:"Format TOML"}),`
`,n(e.p,{get children(){return["Initialize the WASM module before calling ",n(e.code,{children:"format"}),":"]}}),`
`,n(t,{code:`import init, { format, type TombiError } from "tombi-wasm-lib";

await init();

const source = \`
[package]
name = "example"
\`;

try {
  const { formatted, diagnostics } = await format(source, "Cargo.toml");

  if (formatted !== undefined) {
    console.log(formatted);
  }

  for (const diagnostic of diagnostics) {
    if (diagnostic.level === "error") {
      console.error(diagnostic.message);
    } else {
      console.warn(diagnostic.message);
    }
  }
} catch (error) {
  console.error((error as TombiError).message);
}`,language:"ts"}),`
`,n(e.p,{get children(){return[n(e.code,{children:"format"}),` resolves to an object containing the formatted TOML when formatting
succeeds and an array of diagnostics. The diagnostics array is empty when none were
reported. When diagnostics prevent formatting, `,n(e.code,{children:"formatted"}),` is omitted.
Configuration and execution errors reject the Promise as `,n(e.code,{children:"TombiError"}),"."]}}),`
`,n(e.h2,{id:"lint-toml",className:"group relative heading-with-anchor",children:"Lint TOML"}),`
`,n(e.p,{get children(){return["Initialize the WASM module before calling ",n(e.code,{children:"lint"}),":"]}}),`
`,n(t,{code:`import init, { lint, type TombiError } from "tombi-wasm-lib";

await init();

const source = \`
[package]
name = "example"
\`;

try {
  const { diagnostics } = await lint(source, "Cargo.toml");

  if (diagnostics.length > 0) {
    for (const diagnostic of diagnostics) {
      const message = \`\${diagnostic.message} (\${diagnostic.range.start.line}:\${diagnostic.range.start.column})\`;

      if (diagnostic.level === "error") {
        console.error(message);
      } else {
        console.warn(message);
      }
    }
  } else {
    console.log("No diagnostics");
  }
} catch (error) {
  // Configuration and execution errors reject the Promise.
  console.error((error as TombiError).message);
}`,language:"ts"}),`
`,n(e.p,{get children(){return[n(e.code,{children:"lint"}),` resolves to an object containing a diagnostics array. The array is empty when
no diagnostics were reported. Configuration and execution errors reject the Promise
as `,n(e.code,{children:"TombiError"}),"."]}}),`
`,n(e.p,{get children(){return["Both functions accept an optional options object as the third argument. ",n(e.code,{children:"config"}),`
replaces the automatically discovered configuration and accepts an object or string:`]}}),`
`,n(t,{code:`const config = {
  content: \`
toml-version = "v1.1.0"

[format.rules]
indent-width = 4
\`,
  path: "/workspace/tombi.toml",
};

await format(source, "Cargo.toml", { config });
await lint(source, "Cargo.toml", { config });

// The configuration content can be passed as a string shorthand. It is
// treated as the content of a virtual \`tombi.toml\`.
await format(source, "Cargo.toml", { config: config.content });`,language:"ts"}),`
`,n(e.p,{get children(){return["When ",n(e.code,{children:"config"}),` is omitted, Tombi uses automatic discovery and falls back to the
default configuration if no configuration file is found.`]}})]}function m(o={}){const{wrapper:e}={...c(),...o.components};return e?n(e,s(o,{get children(){return n(i,o)}})):i(o)}function a(o,e){throw new Error("Expected component `"+o+"` to be defined: you likely forgot to import, pass, or provide it.")}export{m as default};
