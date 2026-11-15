import{M as d}from"./index-6OQr06v0.js";import{W as l,T as s,N as t}from"./Highlight-CMhf5ZLw.js";import{b as e,k as h}from"./web-DCz9_uPG.js";function c(i){const n={a:"a",code:"code",em:"em",h1:"h1",h2:"h2",h3:"h3",li:"li",ol:"ol",p:"p",strong:"strong",ul:"ul",...d(),...i.components},{CodeBlock:r,PageHeading:o}=n;return r||a("CodeBlock"),o||a("PageHeading"),[e(o,{title:"Configuration",description:"Configure Tombi using tombi.toml or pyproject.toml.",og_url:"https://tombi-toml.github.io/tombi/main/docs/configuration"}),`
`,`
`,e(n.h1,{id:"configuration",className:"group relative heading-with-anchor",children:"Configuration"}),`
`,e(n.p,{get children(){return["Learn how to configure Tombi using the ",e(n.code,{children:"tombi.toml"})," configuration file."]}}),`
`,e(n.p,{get children(){return["Tombi uses a TOML configuration file named ",e(n.code,{children:"tombi.toml"})," (or ",e(n.code,{children:".tombi.toml"}),`).
It also supports `,e(n.code,{children:"[tool.tombi]"})," in ",e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(n.h2,{id:"search-priority",className:"group relative heading-with-anchor",children:"Search Priority"}),`
`,e(n.h3,{id:"project-level",className:"group relative heading-with-anchor",children:"Project Level"}),`
`,e(n.p,{children:"For each directory from the current directory up to the filesystem root:"}),`
`,e(n.ol,{get children(){return[`
`,e(n.li,{get children(){return e(n.code,{children:".tombi.toml"})}}),`
`,e(n.li,{get children(){return e(n.code,{children:"tombi.toml"})}}),`
`,e(n.li,{get children(){return[e(n.code,{children:".config/tombi.toml"})," (see ",e(n.a,{href:"https://github.com/pi0/config-dir",children:"config-dir"})," or ",e(n.a,{href:"https://dot-config.github.io/",children:"dot-config.github.io"}),")"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"[tool.tombi]"})," in ",e(n.code,{children:"pyproject.toml"})]}}),`
`]}}),`
`,e(n.h3,{id:"user-level",className:"group relative heading-with-anchor",children:"User Level"}),`
`,e(n.p,{children:"If nothing is found, Tombi then falls back to:"}),`
`,e(n.ol,{start:"5",get children(){return[`
`,e(n.li,{get children(){return e(n.code,{children:"$XDG_CONFIG_HOME/tombi/config.toml"})}}),`
`,e(n.li,{get children(){return e(n.code,{children:"~/.config/tombi/config.toml"})}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"~/Library/Application Support/tombi/config.toml"})," (macOS), ",e(n.code,{children:"%APPDATA%\\tombi\\config.toml"})," (Windows)"]}}),`
`]}}),`
`,e(n.h3,{id:"system-level",className:"group relative heading-with-anchor",children:"System Level"}),`
`,e(n.ol,{start:"8",get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"/etc/tombi/config.toml"})," (Linux)"]}}),`
`]}}),`
`,e(n.h3,{id:"default-level",className:"group relative heading-with-anchor",children:"Default Level"}),`
`,e(n.ol,{start:"9",get children(){return[`
`,e(n.li,{get children(){return["editor settings via ",e(n.a,{href:"https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_didChangeConfiguration",children:"workspace/didChangeConfiguration"})," (LSP only)"]}}),`
`]}}),`
`,e(l,{get children(){return[e(n.p,{children:"The base directory for configuration search depends on the context: for CLI usage, it is the directory where the command is executed; for LSP usage, it is the directory of the currently opened file."}),e(n.p,{get children(){return["Therefore, it is recommended to place ",e(n.code,{children:"tombi.toml"})," only at the root of your project. If multiple ",e(n.code,{children:"tombi.toml"})," files exist at different levels within a single project, there may be inconsistencies in formatting or diagnostic results between the CLI and the editor."]}})]}}),`
`,e(n.h2,{id:"available-options",className:"group relative heading-with-anchor",children:"Available Options"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#extends",children:"extends"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#toml-version",children:"toml-version"})}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#files",children:"files"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#files-include",children:"files.include"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#files-exclude",children:"files.exclude"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#files-respect-ignore-files",children:"files.respect-ignore-files"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#format",children:"format"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#format-rules",children:"format.rules"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-array-bracket-space-width",children:"format.rules.array-bracket-space-width"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-array-comma-space-width",children:"format.rules.array-comma-space-width"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-comment-style",children:"format.rules.comment-style"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-date-time-delimiter",children:"format.rules.date-time-delimiter"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-group-blank-lines-limit",children:"format.rules.group-blank-lines-limit"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-indent-style",children:"format.rules.indent-style"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-indent-sub-tables",children:"format.rules.indent-sub-tables"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-indent-table-key-value-pairs",children:"format.rules.indent-table-key-value-pairs"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-indent-width",children:"format.rules.indent-width"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-inline-table-brace-space-width",children:"format.rules.inline-table-brace-space-width"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-inline-table-comma-space-width",children:"format.rules.inline-table-comma-space-width"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-key-quote-style",children:"format.rules.key-quote-style"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-key-value-equals-sign-alignment",children:"format.rules.key-value-equals-sign-alignment"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-key-value-equals-sign-space-width",children:"format.rules.key-value-equals-sign-space-width"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-line-ending",children:"format.rules.line-ending"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-line-width",children:"format.rules.line-width"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-string-quote-style",children:"format.rules.string-quote-style"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-table-blank-lines",children:"format.rules.table-blank-lines"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-trailing-comment-alignment",children:"format.rules.trailing-comment-alignment"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#format-rules-trailing-comment-space-width",children:"format.rules.trailing-comment-space-width"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lint",children:"lint"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#lint-rules",children:"lint.rules"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lint-rules-key-empty",children:"lint.rules.key-empty"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#lint-rules-dotted-keys-out-of-order",children:"lint.rules.dotted-keys-out-of-order"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#lint-rules-tables-out-of-order",children:"lint.rules.tables-out-of-order"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp",children:"lsp"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-code-action",children:"lsp.code-action"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-code-action-enabled",children:"lsp.code-action.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-completion",children:"lsp.completion"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-completion-enabled",children:"lsp.completion.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-diagnostic",children:"lsp.diagnostic"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-diagnostic-enabled",children:"lsp.diagnostic.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-document-link",children:"lsp.document-link"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-document-link-enabled",children:"lsp.document-link.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-formatting",children:"lsp.formatting"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-formatting-enabled",children:"lsp.formatting.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-goto-declaration",children:"lsp.goto-declaration"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-goto-declaration-enabled",children:"lsp.goto-declaration.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-goto-definition",children:"lsp.goto-definition"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-goto-definition-enabled",children:"lsp.goto-definition.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-goto-type-definition",children:"lsp.goto-type-definition"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-goto-type-definition-enabled",children:"lsp.goto-type-definition.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-hover",children:"lsp.hover"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-hover-enabled",children:"lsp.hover.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-references",children:"lsp.references"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-references-enabled",children:"lsp.references.enabled"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#lsp-workspace-diagnostic",children:"lsp.workspace-diagnostic"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#lsp-workspace-diagnostic-enabled",children:"lsp.workspace-diagnostic.enabled"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schema",children:"schema"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schema-enabled",children:"schema.enabled"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#schema-strict",children:"schema.strict"})}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schema-format",children:"schema.format"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#schema-format-rules",children:"schema.format.rules"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schema-format-rules-array-values-order",children:"schema.format.rules.array-values-order"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#schema-format-rules-table-keys-order",children:"schema.format.rules.table-keys-order"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schema-lint",children:"schema.lint"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#schema-lint-rules",children:"schema.lint.rules"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schema-lint-rules-deprecated",children:"schema.lint.rules.deprecated"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schema-catalog",children:"schema.catalog"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schema-catalog-paths",children:"schema.catalog.paths"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas",children:"schemas"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#root-schema",children:"schemas[*].toml-version"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#sub-schema",children:"schemas[*].root"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#root-schema",children:"schemas[*].path"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#root-schema",children:"schemas[*].include"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#root-schema",children:"schemas[*].exclude"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#root-schema",children:"schemas[*].strict"})}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas-format",children:"schemas[*].format"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas-format-rules",children:"schemas[*].format.rules"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schemas-format-rules-array-values-order",children:"schemas[*].format.rules.array-values-order"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#schemas-format-rules-table-keys-order",children:"schemas[*].format.rules.table-keys-order"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas-lint",children:"schemas[*].lint"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas-lint-rules",children:"schemas[*].lint.rules"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schemas-lint-rules-deprecated",children:"schemas[*].lint.rules.deprecated"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas-overrides",children:"schemas[*].overrides"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schemas-overrides-targets",children:"schemas[*].overrides[*].targets"})}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas-overrides-format",children:"schemas[*].overrides[*].format"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas-overrides-format-rules",children:"schemas[*].overrides[*].format.rules"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schemas-overrides-format-rules-array-values-order",children:"schemas[*].overrides[*].format.rules.array-values-order"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#schemas-overrides-format-rules-table-keys-order",children:"schemas[*].overrides[*].format.rules.table-keys-order"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas-overrides-lint",children:"schemas[*].overrides[*].lint"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#schemas-overrides-lint-rules",children:"schemas[*].overrides[*].lint.rules"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#schemas-overrides-lint-rules-deprecated",children:"schemas[*].overrides[*].lint.rules.deprecated"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#overrides",children:"overrides"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#overrides-files",children:"overrides[*].files"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#overrides-files-include",children:"overrides[*].files.include"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#overrides-files-exclude",children:"overrides[*].files.exclude"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#overrides-format",children:"overrides[*].format"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#overrides-format-enabled",children:"overrides[*].format.enabled"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#overrides-format-rules",children:"overrides[*].format.rules"})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#overrides-lint",children:"overrides[*].lint"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#overrides-lint-enabled",children:"overrides[*].lint.enabled"})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#overrides-lint-rules",children:"overrides[*].lint.rules"})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions",children:"extensions"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-cargo",children:'extensions."tombi-toml/cargo"'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-cargo-lint",children:'extensions."tombi-toml/cargo".lint'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lint-rules",children:'extensions."tombi-toml/cargo".lint.rules'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lint-rules-missing-build-script",children:'extensions."tombi-toml/cargo".lint.rules.missing-build-script'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lint-rules-missing-license-file",children:'extensions."tombi-toml/cargo".lint.rules.missing-license-file'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lint-rules-missing-path-dependency",children:'extensions."tombi-toml/cargo".lint.rules.missing-path-dependency'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lint-rules-missing-readme",children:'extensions."tombi-toml/cargo".lint.rules.missing-readme'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lint-rules-missing-target-path",children:'extensions."tombi-toml/cargo".lint.rules.missing-target-path'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lint-rules-missing-workspace",children:'extensions."tombi-toml/cargo".lint.rules.missing-workspace'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lint-rules-missing-workspace-member",children:'extensions."tombi-toml/cargo".lint.rules.missing-workspace-member'})}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-cargo-lsp",children:'extensions."tombi-toml/cargo".lsp'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lsp-code-action",children:'extensions."tombi-toml/cargo".lsp.code-action'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lsp-completion",children:'extensions."tombi-toml/cargo".lsp.completion'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lsp-document-link",children:'extensions."tombi-toml/cargo".lsp.document-link'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lsp-goto-declaration",children:'extensions."tombi-toml/cargo".lsp.goto-declaration'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lsp-goto-definition",children:'extensions."tombi-toml/cargo".lsp.goto-definition'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lsp-hover",children:'extensions."tombi-toml/cargo".lsp.hover'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lsp-inlay-hint",children:'extensions."tombi-toml/cargo".lsp.inlay-hint'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-cargo-lsp-references",children:'extensions."tombi-toml/cargo".lsp.references'})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-pyproject",children:'extensions."tombi-toml/pyproject"'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-pyproject-lint",children:'extensions."tombi-toml/pyproject".lint'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules",children:'extensions."tombi-toml/pyproject".lint.rules'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-ambiguous-import-name",children:'extensions."tombi-toml/pyproject".lint.rules.ambiguous-import-name'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-cyclic-include-group",children:'extensions."tombi-toml/pyproject".lint.rules.cyclic-include-group'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-duplicate-dependency-group",children:'extensions."tombi-toml/pyproject".lint.rules.duplicate-dependency-group'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-duplicate-extra-name",children:'extensions."tombi-toml/pyproject".lint.rules.duplicate-extra-name'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-duplicate-import-name",children:'extensions."tombi-toml/pyproject".lint.rules.duplicate-import-name'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-invalid-dependency-specifier",children:'extensions."tombi-toml/pyproject".lint.rules.invalid-dependency-specifier'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-invalid-email-display-name",children:'extensions."tombi-toml/pyproject".lint.rules.invalid-email-display-name'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-invalid-entry-point-target",children:'extensions."tombi-toml/pyproject".lint.rules.invalid-entry-point-target'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-invalid-import-name",children:'extensions."tombi-toml/pyproject".lint.rules.invalid-import-name'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-missing-import-name-parent",children:'extensions."tombi-toml/pyproject".lint.rules.missing-import-name-parent'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-undefined-include-group",children:'extensions."tombi-toml/pyproject".lint.rules.undefined-include-group'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lint-rules-unknown-classifier",children:'extensions."tombi-toml/pyproject".lint.rules.unknown-classifier'})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-pyproject-lsp",children:'extensions."tombi-toml/pyproject".lsp'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lsp-code-action",children:'extensions."tombi-toml/pyproject".lsp.code-action'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lsp-completion",children:'extensions."tombi-toml/pyproject".lsp.completion'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lsp-document-link",children:'extensions."tombi-toml/pyproject".lsp.document-link'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lsp-goto-declaration",children:'extensions."tombi-toml/pyproject".lsp.goto-declaration'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lsp-goto-definition",children:'extensions."tombi-toml/pyproject".lsp.goto-definition'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lsp-hover",children:'extensions."tombi-toml/pyproject".lsp.hover'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lsp-inlay-hint",children:'extensions."tombi-toml/pyproject".lsp.inlay-hint'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-pyproject-lsp-references",children:'extensions."tombi-toml/pyproject".lsp.references'})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-tombi",children:'extensions."tombi-toml/tombi"'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-tombi-lint",children:'extensions."tombi-toml/tombi".lint'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-tombi-lint-rules",children:'extensions."tombi-toml/tombi".lint.rules'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-tombi-lint-rules-missing-catalog-path",children:'extensions."tombi-toml/tombi".lint.rules.missing-catalog-path'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-tombi-lint-rules-missing-extends-path",children:'extensions."tombi-toml/tombi".lint.rules.missing-extends-path'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-tombi-lint-rules-missing-schema-path",children:'extensions."tombi-toml/tombi".lint.rules.missing-schema-path'})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.li,{get children(){return[e(n.a,{href:"#extensions-tombi-toml-tombi-lsp",children:'extensions."tombi-toml/tombi".lsp'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-tombi-lsp-completion",children:'extensions."tombi-toml/tombi".lsp.completion'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-tombi-lsp-goto-definition",children:'extensions."tombi-toml/tombi".lsp.goto-definition'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-tombi-lsp-document-link",children:'extensions."tombi-toml/tombi".lsp.document-link'})}}),`
`,e(n.li,{get children(){return e(n.a,{href:"#extensions-tombi-toml-tombi-lsp-hover",children:'extensions."tombi-toml/tombi".lsp.hover'})}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.h2,{id:"full-structure",className:"group relative heading-with-anchor",children:"Full Structure"}),`
`,e(r,{code:`extends = []
toml-version = "v1.0.0"

[files]
include = ["**/*.toml"]
exclude = [".cache/**/*.toml"]
respect-ignore-files = true

[format]
[format.rules]
array-bracket-space-width = 0
array-comma-space-width = 1
comment-style = "normalize"
date-time-delimiter = "T"
group-blank-lines-limit = 1
indent-style = "space"
indent-sub-tables = false
indent-table-key-value-pairs = false
indent-width = 2
inline-table-brace-space-width = 1
inline-table-comma-space-width = 1
key-quote-style = "double"
key-value-equals-sign-alignment = false
key-value-equals-sign-space-width = 1
line-ending = "lf"
string-quote-style = "double"
table-blank-lines = 1
trailing-comment-alignment = false
trailing-comment-space-width = 2

[lint]
[lint.rules]
dotted-keys-out-of-order = "warn"
key-empty = "warn"
tables-out-of-order = "warn"

[lsp]
code-action.enabled = true
completion.enabled = true
diagnostic.enabled = true
document-link.enabled = true
formatting.enabled = true
goto-declaration.enabled = true
goto-definition.enabled = true
goto-type-definition.enabled = true
hover.enabled = true
workspace-diagnostic.enabled = true

[schema]
enabled = true
strict = true
catalog = {
  paths = [
    "tombi://www.schemastore.org/api/json/catalog.json",
    "https://www.schemastore.org/api/json/catalog.json",
  ],
}
format.rules = {
  array-values-order.enabled = true,
  table-keys-order.enabled = true,
}
lint.rules = {
  deprecated = "warn",
}

# Root Schema
[[schemas]]
toml-version = "v1.0.0"
path = "https://example.com/schema.json"
include = ["example.toml"]
strict = false
format.rules = {
  array-values-order.enabled = true,
  table-keys-order.enabled = true,
}
lint.rules = {
  deprecated = "warn",
}
overrides = [
  {
    targets = ["", "tool.*"],
    format.rules.array-values-order = "ascending",
    format.rules.table-keys-order = "ascending",
    lint.rules.deprecated = "warn",
  },
]

# Sub Schema
[[schemas]]
root = "tool.taskipy"
path = "schemas/partial-taskipy.schema.json"
include = ["pyproject.toml"]
lint.rules = {
  deprecated = "warn",
}

# Override config
[[overrides]]
files = {
  include = ["specific/**/*.toml"],
  exclude = ["specific/**/ignored.toml"],
}
format = {
  enabled = true,
  rules = {
    array-bracket-space-width = 1,
    array-comma-space-width = 1,
    date-time-delimiter = "space",
    group-blank-lines-limit = 1,
    indent-style = "tab",
    indent-sub-tables = true,
    indent-table-key-value-pairs = true,
    indent-width = 4,
    inline-table-brace-space-width = 0,
    inline-table-comma-space-width = 1,
    key-quote-style = "single",
    key-value-equals-sign-alignment = true,
    key-value-equals-sign-space-width = 1,
    line-ending = "crlf",
    line-width = 120,
    string-quote-style = "single",
    table-blank-lines = 1,
    trailing-comment-alignment = true,
    trailing-comment-space-width = 1,
  },
}
lint = {
  enabled = true,
  rules = {
    dotted-keys-out-of-order = "error",
    key-empty = "error",
    tables-out-of-order = "error",
  },
}

[extensions]
"tombi-toml/cargo" = {
  lsp = {
    code-action = {
      add-to-workspace-and-inherit-dependency.enabled = true,
      convert-dependency-to-table-format.enabled = true,
      inherit-dependency-from-workspace.enabled = true,
      inherit-from-workspace.enabled = true,
      update-dependency-to-latest-version.enabled = true,
    },
    completion = {
      dependency-feature.enabled = true,
      dependency-version.enabled = true,
      path.enabled = true,
    },
    document-link = {
      crates-io.enabled = true,
    },
    goto-declaration = {
      dependency.enabled = true,
    },
    goto-definition = {
      dependency.enabled = true,
      member.enabled = true,
      path.enabled = true,
    },
    hover = {
      default-features.enabled = true,
      dependency-detail.enabled = true,
      feature-dependencies.enabled = true,
    },
    inlay-hint = {
      default-features.enabled = true,
      dependency-version.enabled = true,
      workspace-value.enabled = true,
    },
  }
}
"tombi-toml/pyproject" = {
  lsp = {
    code-action = {
      add-to-workspace-and-use-workspace-dependency.enabled = true,
      update-dependency-to-latest-version.enabled = true,
      use-workspace-dependency.enabled = true,
    },
    completion = {
      path.enabled = true,
    },
    document-link = {
      pypi-org.enabled = true,
    },
    goto-declaration = {
      dependency.enabled = true,
      member.enabled = true,
    },
    goto-definition = {
      dependency.enabled = true,
      member.enabled = true,
      path.enabled = true,
    },
    hover = {
      dependency-detail.enabled = true,
    },
    inlay-hint = {
      dependency-version.enabled = true,
    },
  }
}
"tombi-toml/tombi" = {
  lsp = {
    completion = {
      path.enabled = true,
    },
    goto-definition = {
      path.enabled = true,
    },
    hover.enabled = true,
  }
}`,language:"toml"}),`
`,e(n.h3,{id:"extends",className:"group relative heading-with-anchor",children:"extends"}),`
`,e(n.p,{get children(){return["Use ",e(n.code,{children:"extends"})," to inherit settings from one or more config files, then put repository-specific values in the local config:"]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"String[]"})," (paths or URLs to ",e(n.code,{children:"tombi.toml"})," or ",e(n.code,{children:".tombi.toml"}),")"]}}),`
`]}}),`
`,e(r,{code:`extends = ["../shared/tombi.toml", "https://example.org/config/tombi.toml"]

[format.rules]
line-width = 100`,language:"toml"}),`
`,e(n.p,{get children(){return["Local paths are resolved relative to the directory of the config file that declares them, or relative to the project root for ",e(n.code,{children:".config/tombi.toml"}),", like the paths of ",e(n.code,{children:"schemas"})," and ",e(n.code,{children:"schema.catalog"}),". HTTP and HTTPS URLs are supported; a relative URL in a remote config is resolved against that URL. A remote config cannot use ",e(n.code,{children:"file://"})," to read a local file. In ",e(n.code,{children:"pyproject.toml"}),", place ",e(n.code,{children:"extends"})," inside ",e(n.code,{children:"[tool.tombi]"}),"."]}}),`
`,e(n.p,{get children(){return["The extended file must be named ",e(n.code,{children:"tombi.toml"})," or ",e(n.code,{children:".tombi.toml"}),", so that the Tombi config schema applies to it. ",e(n.code,{children:"pyproject.toml"})," cannot be extended."]}}),`
`,e(n.p,{get children(){return["In the editor, an ",e(n.code,{children:"extends"})," entry that does not exist, cannot be loaded (for example a config cycle, a parse error in an inherited config, or a failed fetch), or uses an unsupported URL is reported as an error, and you can jump to the extended config with go to definition."]}}),`
`,e(n.p,{get children(){return["The configs in ",e(n.code,{children:"extends"})," are applied in the order listed, starting with the first entry. The local config is applied last, and later configs take precedence over earlier ones. Tables merge recursively, and local scalar values replace inherited values. Most arrays are replaced by the higher-priority config. ",e(n.code,{children:"schemas"})," and ",e(n.code,{children:"overrides"})," are combined with higher-priority entries first, so local entries take precedence over inherited entries."]}}),`
`,e(n.p,{children:`Paths and globs in inherited settings are resolved relative to the extending config's base directory, using Tombi's existing path rules.
An inheritance chain can follow at most 32 references. One resolution can use at most 256 references in total.`}),`
`,e(n.h3,{id:"toml-version",className:"group relative heading-with-anchor",children:"toml-version"}),`
`,e(n.p,{children:"Specifies the TOML version to use if not specified in the schema and comment directive."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"v1.0.0" | "v1.1.0"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"v1.0.0"'})]}}),`
`]}}),`
`,e(n.h3,{id:"files",className:"group relative heading-with-anchor",children:"files"}),`
`,e(n.p,{children:"Configure file patterns for formatting and linting operations."}),`
`,e(n.h3,{id:"files-include",className:"group relative heading-with-anchor",children:"files.include"}),`
`,e(n.p,{children:"The file match pattern to include in formatting and linting. Supports glob pattern."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"String[]"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'["**/*.toml"]'})]}}),`
`]}}),`
`,e(n.h3,{id:"files-exclude",className:"group relative heading-with-anchor",children:"files.exclude"}),`
`,e(n.p,{children:"The file match pattern to exclude from formatting and linting. Supports glob pattern."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"String[]"})]}}),`
`]}}),`
`,e(n.h3,{id:"files-respect-ignore-files",className:"group relative heading-with-anchor",children:"files.respect-ignore-files"}),`
`,e(n.p,{get children(){return["Whether to respect ",e(n.code,{children:".ignore"}),", ",e(n.code,{children:".gitignore"}),", and ",e(n.code,{children:".git/info/exclude"})," while discovering files."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"format",className:"group relative heading-with-anchor",children:"format"}),`
`,e(n.p,{children:"Formatter options for TOML files."}),`
`,e(n.h3,{id:"format-rules",className:"group relative heading-with-anchor",children:"format.rules"}),`
`,e(n.p,{children:"Configure formatting styles."}),`
`,e(s,{get children(){return[e(n.p,{children:"If you want to know Tombi's recommended formatting style, simply use the default settings."}),e(n.p,{get children(){return["Originally, Tombi enforced strict formatting rules similar to ",e(n.a,{href:"https://github.com/psf/black",children:"black"}),`,
but we relaxed some requirements to welcome Taplo users.
Even so, we still encourage you to stick with the defaults to avoid wasting time on style debates and maintain consistency.`]}})]}}),`
`,e(n.h3,{id:"format-rules-array-bracket-space-width",className:"group relative heading-with-anchor",children:"format.rules.array-bracket-space-width"}),`
`,e(n.p,{children:"The number of spaces inside the brackets of a single line array."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"0"})]}}),`
`]}}),`
`,e(r,{code:`key = [ 1, 2, 3 ]
#      ^       ^  <- this`,language:"toml"}),`
`,e(n.h3,{id:"format-rules-array-comma-space-width",className:"group relative heading-with-anchor",children:"format.rules.array-comma-space-width"}),`
`,e(n.p,{children:"The number of spaces after the comma in a single line array."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"1"})]}}),`
`]}}),`
`,e(r,{code:`key = [ 1, 2, 3 ]
#         ^  ^  <- this`,language:"toml"}),`
`,e(n.h3,{id:"format-rules-comment-style",className:"group relative heading-with-anchor",children:"format.rules.comment-style"}),`
`,e(n.p,{children:"The style used to format comments."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"normalize" | "preserve"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"normalize"'})]}}),`
`,e(n.li,{get children(){return["Enum Values:",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"normalize"}),": Normalize comment text according to Tombi's formatting rules."]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"preserve"}),": Preserve the original comment text while formatting its placement normally."]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.p,{children:"For example, comment indentation and the spacing before trailing comments are still controlled by the formatter."}),`
`,e(n.h3,{id:"format-rules-date-time-delimiter",className:"group relative heading-with-anchor",children:"format.rules.date-time-delimiter"}),`
`,e(n.p,{children:"The delimiter between date and time"}),`
`,e(n.p,{get children(){return["In accordance with ",e(n.a,{href:"https://datatracker.ietf.org/doc/html/rfc3339",children:"RFC 3339"}),", you can use ",e(n.code,{children:"T"})," or space character between date and time."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"T" | "space" | "preserve"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"T"'})]}}),`
`,e(n.li,{get children(){return["Enum Values:",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"T"}),": Use ",e(n.code,{children:"T"})," between date and time like ",e(n.code,{children:"2001-01-01T00:00:00"})]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"space"}),": Use space between date and time like ",e(n.code,{children:"2001-01-01 00:00:00"})]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"preserve"}),": Preserve the original delimiter."]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.h3,{id:"format-rules-group-blank-lines-limit",className:"group relative heading-with-anchor",children:"format.rules.group-blank-lines-limit"}),`
`,e(n.p,{children:"The maximum number of blank lines to preserve between groups."}),`
`,e(n.p,{children:"These boundaries remain significant for auto sorting. The formatter only caps the amount of spacing that is preserved."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"1"})]}}),`
`,e(n.li,{get children(){return["Minimum: ",e(n.code,{children:"1"})]}}),`
`]}}),`
`,e(r,{code:`# BEFORE
key1 = "value1"
key2 = "value2"

key3 = "value3"


key4 = "value4"



key5 = "value5"

# AFTER (\`group-blank-lines-limit = 2\`)
key1 = "value1"
key2 = "value2"

key3 = "value3"


key4 = "value4"


key5 = "value5"`,language:"toml"}),`
`,e(n.h3,{id:"format-rules-indent-style",className:"group relative heading-with-anchor",children:"format.rules.indent-style"}),`
`,e(n.p,{children:"The style of indentation"}),`
`,e(n.p,{children:"Whether to use spaces or tabs for indentation."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"space" | "tab"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"space"'})]}}),`
`,e(n.li,{get children(){return["Enum Values:",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"space"}),": Use spaces for indentation."]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"tab"}),": Use tabs for indentation."]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.h3,{id:"format-rules-indent-sub-tables",className:"group relative heading-with-anchor",children:"format.rules.indent-sub-tables"}),`
`,e(n.p,{children:"Whether to indent the sub-tables."}),`
`,e(n.p,{get children(){return["If ",e(n.code,{children:"true"}),", the sub-table will be indented."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"false"})]}}),`
`]}}),`
`,e(r,{code:`[table]
    [table.sub-table]
    key = "value"
# ^^  <- this`,language:"toml"}),`
`,e(n.h3,{id:"format-rules-indent-table-key-value-pairs",className:"group relative heading-with-anchor",children:"format.rules.indent-table-key-value-pairs"}),`
`,e(n.p,{children:"Whether to indent the table key-value pairs."}),`
`,e(n.p,{get children(){return["If ",e(n.code,{children:"true"}),", the table key-value pairs will be indented."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"false"})]}}),`
`]}}),`
`,e(r,{code:`[table]
    key = "value"
# ^^  <- this`,language:"toml"}),`
`,e(n.h3,{id:"format-rules-indent-width",className:"group relative heading-with-anchor",children:"format.rules.indent-width"}),`
`,e(n.p,{children:"The number of spaces per indentation level."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"2"})]}}),`
`]}}),`
`,e(l,{get children(){return e(n.p,{get children(){return["This option is only used when the indentation style is ",e(n.code,{children:"space"}),"."]}})}}),`
`,e(n.h3,{id:"format-rules-inline-table-brace-space-width",className:"group relative heading-with-anchor",children:"format.rules.inline-table-brace-space-width"}),`
`,e(n.p,{children:"The number of spaces inside the brackets of a single line inline table."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"1"})]}}),`
`]}}),`
`,e(r,{code:`key = { a = 1, b = 2 }
#      ^            ^  <- this`,language:"toml"}),`
`,e(n.h3,{id:"format-rules-inline-table-comma-space-width",className:"group relative heading-with-anchor",children:"format.rules.inline-table-comma-space-width"}),`
`,e(n.p,{children:"The number of spaces after the comma in a single line inline table."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"1"})]}}),`
`]}}),`
`,e(r,{code:`key = { a = 1, b = 2 }
#             ^  <- this`,language:"toml"}),`
`,e(n.h3,{id:"format-rules-key-quote-style",className:"group relative heading-with-anchor",children:"format.rules.key-quote-style"}),`
`,e(n.p,{children:"The preferred quote character for quoted keys."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"double" | "single" | "preserve"'})]}}),`
`,e(n.li,{get children(){return["Default: uses ",e(n.a,{href:"#format-rules-string-quote-style",children:"string-quote-style"})]}}),`
`,e(n.li,{get children(){return["Enum Values:",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"double"}),": Prefer the double quote like ",e(n.code,{children:'"key"'})]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"single"}),": Prefer the single quote like ",e(n.code,{children:"'key'"})]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"preserve"}),": Preserve the original quote"]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.h3,{id:"format-rules-key-value-equals-sign-alignment",className:"group relative heading-with-anchor",children:"format.rules.key-value-equals-sign-alignment"}),`
`,e(n.p,{children:"Whether to align the equals sign in the key-value pairs."}),`
`,e(n.p,{get children(){return["If ",e(n.code,{children:"true"}),", the equals sign in the key-value pairs will be aligned."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"false"})]}}),`
`]}}),`
`,e(r,{code:`# BEFORE
key = "value1"
key2 = "value2"
key3.key4 = "value3"

# AFTER
key       = "value1"
key2      = "value2"
key3.key4 = "value3"`,language:"toml"}),`
`,e(l,{get children(){return e(n.p,{get children(){return["This feature does ",e(n.strong,{children:"not"})," apply to key-value pairs inside single line inline tables."]}})}}),`
`,e(n.h3,{id:"format-rules-key-value-equals-sign-space-width",className:"group relative heading-with-anchor",children:"format.rules.key-value-equals-sign-space-width"}),`
`,e(n.p,{children:"The number of spaces around the equals sign in a key-value pair."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"1"})]}}),`
`]}}),`
`,e(r,{code:`key = "value"
#  ^ ^  <- this`,language:"toml"}),`
`,e(n.h3,{id:"format-rules-line-ending",className:"group relative heading-with-anchor",children:"format.rules.line-ending"}),`
`,e(n.p,{children:"The type of line ending."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"lf" | "crlf"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"lf"'})]}}),`
`,e(n.li,{get children(){return["Enum Values:",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"lf"}),": Line Feed only (",e(n.code,{children:"\\n"}),"), common on Linux and macOS as well as inside git repos."]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"crlf"}),": Carriage Return Line Feed (",e(n.code,{children:"\\r\\n"}),"), common on Windows."]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.h3,{id:"format-rules-line-width",className:"group relative heading-with-anchor",children:"format.rules.line-width"}),`
`,e(n.p,{children:"The maximum line width. If omitted, the formatter does not apply a line-width limit."}),`
`,e(n.p,{children:`When specified, the formatter will try to keep lines within this width. Explicit multiline
hints, such as a trailing comma, are still respected without a line-width setting.`}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{children:"Default: no limit"}),`
`]}}),`
`,e(n.h3,{id:"format-rules-string-quote-style",className:"group relative heading-with-anchor",children:"format.rules.string-quote-style"}),`
`,e(n.p,{children:"The preferred quote character for strings."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"double" | "single" | "preserve"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"double"'})]}}),`
`,e(n.li,{get children(){return["Enum Values:",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"double"}),": Prefer the double quote like ",e(n.code,{children:'"value"'})]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"single"}),": Prefer the single quote like ",e(n.code,{children:"'value'"})]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"preserve"}),": Preserve the original quote"]}}),`
`]}}),`
`]}}),`
`]}}),`
`,e(n.h3,{id:"format-rules-table-blank-lines",className:"group relative heading-with-anchor",children:"format.rules.table-blank-lines"}),`
`,e(n.p,{children:"The number of blank lines inserted between table or array-of-table blocks when they are separated."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"1"})]}}),`
`,e(n.li,{get children(){return["Minimum: ",e(n.code,{children:"0"})]}}),`
`]}}),`
`,e(r,{code:`# BEFORE
key1 = "value1"
[aaa]
key2 = "value2"
[bbb]
key3 = "value3"

# AFTER (\`table-blank-lines = 2\`)
key1 = "value1"


[aaa]
key2 = "value2"


[bbb]
key3 = "value3"`,language:"toml"}),`
`,e(n.p,{children:"Tight parent/child table adjacency is controlled separately, so this does not apply when a child table follows a parent table that has no key-value pairs."}),`
`,e(r,{code:`# BEFORE
[aaa]

[aaa.bbb]

# AFTER
[aaa]
[aaa.bbb]`,language:"toml"}),`
`,e(n.h3,{id:"format-rules-trailing-comment-alignment",className:"group relative heading-with-anchor",children:"format.rules.trailing-comment-alignment"}),`
`,e(n.p,{children:"Whether to align the trailing comments in the key-value pairs."}),`
`,e(n.p,{get children(){return["If ",e(n.code,{children:"true"}),", the trailing comments in value/key-value pairs will be aligned."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"false"})]}}),`
`]}}),`
`,e(r,{code:`# BEFORE
key = "value1"  # comment 1
key2 = "value2"  # comment 2
key3.key4 = "value3"  # comment 3

# AFTER
key = "value1"        # comment 1
key2 = "value2"       # comment 2
key3.key4 = "value3"  # comment 3`,language:"toml"}),`
`,e(t,{get children(){return e(n.p,{children:"The trailing comments of table header are not targeted by alignment."})}}),`
`,e(n.h3,{id:"format-rules-trailing-comment-space-width",className:"group relative heading-with-anchor",children:"format.rules.trailing-comment-space-width"}),`
`,e(n.p,{children:"The number of spaces before the trailing comment."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Number"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"2"})]}}),`
`]}}),`
`,e(r,{code:`key = "value"  # trailing comment
#            ^^  <- this`,language:"toml"}),`
`,e(n.h3,{id:"lint",className:"group relative heading-with-anchor",children:"lint"}),`
`,e(n.p,{children:"Linter options for TOML files."}),`
`,e(n.h3,{id:"lint-rules",className:"group relative heading-with-anchor",children:"lint.rules"}),`
`,e(n.p,{children:"Configure individual lint rules with severity levels."}),`
`,e(n.h3,{id:"lint-rules-key-empty",className:"group relative heading-with-anchor",children:"lint.rules.key-empty"}),`
`,e(n.p,{children:"Check if the key is empty."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"warn"'})]}}),`
`]}}),`
`,e(r,{code:`# VALID BUT DISCOURAGED
"" = true`,language:"toml"}),`
`,e(t,{get children(){return e(n.p,{get children(){return["This rule is disabled when the JSON Schema for the table specifies ",e(n.code,{children:"propertyNames.minLength: 0"}),", since the schema explicitly allows empty keys."]}})}}),`
`,e(n.h3,{id:"lint-rules-dotted-keys-out-of-order",className:"group relative heading-with-anchor",children:"lint.rules.dotted-keys-out-of-order"}),`
`,e(n.p,{children:"Check if dotted keys are defined out of order."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"warn"'})]}}),`
`]}}),`
`,e(r,{code:`# VALID BUT DISCOURAGED
apple.type = "fruit"
orange.type = "fruit"
apple.skin = "thin"
orange.skin = "thick"

# RECOMMENDED
apple.type = "fruit"
apple.skin = "thin"
orange.type = "fruit"
orange.skin = "thick"`,language:"toml"}),`
`,e(n.h3,{id:"lint-rules-tables-out-of-order",className:"group relative heading-with-anchor",children:"lint.rules.tables-out-of-order"}),`
`,e(n.p,{children:"Check if tables are defined out of order."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"warn"'})]}}),`
`]}}),`
`,e(r,{code:`# VALID BUT DISCOURAGED
[fruit.apple]
[animal]
[fruit.orange]

# RECOMMENDED
[fruit.apple]
[fruit.orange]
[animal]`,language:"toml"}),`
`,e(n.h3,{id:"lsp",className:"group relative heading-with-anchor",children:"lsp"}),`
`,e(n.p,{children:"Language Server Protocol (LSP) feature configuration. Each feature can be individually enabled or disabled."}),`
`,e(n.h3,{id:"lsp-code-action",className:"group relative heading-with-anchor",children:"lsp.code-action"}),`
`,e(n.p,{children:"Configure code action feature."}),`
`,e(n.h3,{id:"lsp-code-action-enabled",className:"group relative heading-with-anchor",children:"lsp.code-action.enabled"}),`
`,e(n.p,{children:"Enable or disable code action feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-completion",className:"group relative heading-with-anchor",children:"lsp.completion"}),`
`,e(n.p,{children:"Configure completion feature."}),`
`,e(n.h3,{id:"lsp-completion-enabled",className:"group relative heading-with-anchor",children:"lsp.completion.enabled"}),`
`,e(n.p,{children:"Enable or disable completion feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-diagnostic",className:"group relative heading-with-anchor",children:"lsp.diagnostic"}),`
`,e(n.p,{children:"Configure diagnostic feature."}),`
`,e(n.h3,{id:"lsp-diagnostic-enabled",className:"group relative heading-with-anchor",children:"lsp.diagnostic.enabled"}),`
`,e(n.p,{children:"Enable or disable diagnostic feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-document-link",className:"group relative heading-with-anchor",children:"lsp.document-link"}),`
`,e(n.p,{children:"Configure document link feature."}),`
`,e(n.h3,{id:"lsp-document-link-enabled",className:"group relative heading-with-anchor",children:"lsp.document-link.enabled"}),`
`,e(n.p,{children:"Enable or disable document link feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-formatting",className:"group relative heading-with-anchor",children:"lsp.formatting"}),`
`,e(n.p,{children:"Configure formatting feature."}),`
`,e(n.h3,{id:"lsp-formatting-enabled",className:"group relative heading-with-anchor",children:"lsp.formatting.enabled"}),`
`,e(n.p,{children:"Enable or disable formatting feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-goto-declaration",className:"group relative heading-with-anchor",children:"lsp.goto-declaration"}),`
`,e(n.p,{children:"Configure go to declaration feature."}),`
`,e(n.h3,{id:"lsp-goto-declaration-enabled",className:"group relative heading-with-anchor",children:"lsp.goto-declaration.enabled"}),`
`,e(n.p,{children:"Enable or disable go to declaration feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-goto-definition",className:"group relative heading-with-anchor",children:"lsp.goto-definition"}),`
`,e(n.p,{children:"Configure go to definition feature."}),`
`,e(n.h3,{id:"lsp-goto-definition-enabled",className:"group relative heading-with-anchor",children:"lsp.goto-definition.enabled"}),`
`,e(n.p,{children:"Enable or disable go to definition feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-goto-type-definition",className:"group relative heading-with-anchor",children:"lsp.goto-type-definition"}),`
`,e(n.p,{children:"Configure go to type definition feature."}),`
`,e(n.h3,{id:"lsp-goto-type-definition-enabled",className:"group relative heading-with-anchor",children:"lsp.goto-type-definition.enabled"}),`
`,e(n.p,{children:"Enable or disable go to type definition feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-hover",className:"group relative heading-with-anchor",children:"lsp.hover"}),`
`,e(n.p,{children:"Configure hover feature."}),`
`,e(n.h3,{id:"lsp-hover-enabled",className:"group relative heading-with-anchor",children:"lsp.hover.enabled"}),`
`,e(n.p,{children:"Enable or disable hover feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-references",className:"group relative heading-with-anchor",children:"lsp.references"}),`
`,e(n.p,{children:"Configure references feature."}),`
`,e(n.h3,{id:"lsp-references-enabled",className:"group relative heading-with-anchor",children:"lsp.references.enabled"}),`
`,e(n.p,{children:"Enable or disable references feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"lsp-workspace-diagnostic",className:"group relative heading-with-anchor",children:"lsp.workspace-diagnostic"}),`
`,e(n.p,{children:"Configure workspace diagnostic feature."}),`
`,e(n.h3,{id:"lsp-workspace-diagnostic-enabled",className:"group relative heading-with-anchor",children:"lsp.workspace-diagnostic.enabled"}),`
`,e(n.p,{children:"Enable or disable workspace diagnostic feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"schema",className:"group relative heading-with-anchor",children:"schema"}),`
`,e(n.p,{children:"JSON Schema validation options for TOML files."}),`
`,e(n.h3,{id:"schema-enabled",className:"group relative heading-with-anchor",children:"schema.enabled"}),`
`,e(n.p,{children:"Enable or disable schema validation."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"schema-strict",className:"group relative heading-with-anchor",children:"schema.strict"}),`
`,e(n.p,{get children(){return["Enable strict schema validation. If ",e(n.code,{children:"additionalProperties"})," is not specified in the JSON Schema, the strict mode treats it as ",e(n.code,{children:"additionalProperties: false"}),", which is different from the JSON Schema specification."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"schema-format",className:"group relative heading-with-anchor",children:"schema.format"}),`
`,e(n.p,{children:"Configure format rules for schema-defined ordering."}),`
`,e(n.h3,{id:"schema-format-rules",className:"group relative heading-with-anchor",children:"schema.format.rules"}),`
`,e(n.p,{get children(){return["Configure global schema format rules. Matching ",e(n.code,{children:"[[schemas]]"})," format rules, ",e(n.code,{children:"[[schemas.overrides]]"}),", and comment directives take precedence over these settings."]}}),`
`,e(n.h3,{id:"schema-format-rules-array-values-order",className:"group relative heading-with-anchor",children:"schema.format.rules.array-values-order"}),`
`,e(n.p,{children:"Configure schema-defined ordering for array values."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"schema-format-rules-array-values-order-enabled",className:"group relative heading-with-anchor",children:"schema.format.rules.array-values-order.enabled"}),`
`,e(n.p,{children:"Enable or disable schema-defined array value ordering for all schemas. This is the fallback when no more specific setting applies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"schema-format-rules-table-keys-order",className:"group relative heading-with-anchor",children:"schema.format.rules.table-keys-order"}),`
`,e(n.p,{children:"Configure schema-defined ordering for table keys."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"schema-format-rules-table-keys-order-enabled",className:"group relative heading-with-anchor",children:"schema.format.rules.table-keys-order.enabled"}),`
`,e(n.p,{children:"Enable or disable schema-defined table key ordering for all schemas. This is the fallback when no more specific setting applies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"schema-lint",className:"group relative heading-with-anchor",children:"schema.lint"}),`
`,e(n.p,{children:"Configure lint rules for values deprecated by the active JSON Schema."}),`
`,e(n.h3,{id:"schema-lint-rules",className:"group relative heading-with-anchor",children:"schema.lint.rules"}),`
`,e(n.p,{children:"Configure global schema lint rules. Schema-specific rules and overrides take precedence over these settings."}),`
`,e(n.h3,{id:"schema-lint-rules-deprecated",className:"group relative heading-with-anchor",children:"schema.lint.rules.deprecated"}),`
`,e(n.p,{get children(){return["Set the severity for values marked as ",e(n.code,{children:"deprecated: true"})," by the active JSON Schema. This is the fallback for all schemas when no more specific deprecated severity is configured."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"warn"'})," when no schema-specific setting applies"]}}),`
`]}}),`
`,e(n.h3,{id:"schema-catalog",className:"group relative heading-with-anchor",children:"schema.catalog"}),`
`,e(n.p,{children:"Configure schema catalog for automatic schema detection."}),`
`,e(n.h3,{id:"schema-catalog-paths",className:"group relative heading-with-anchor",children:"schema.catalog.paths"}),`
`,e(n.p,{get children(){return["The schema catalog path/url array. The catalog is evaluated after the schemas specified by ",e(n.code,{children:"[[schemas]]"}),". Schemas are loaded in order from the beginning of the catalog list."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"String[]"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'["tombi://www.schemastore.org/api/json/catalog.json", "https://www.schemastore.org/api/json/catalog.json"]'})]}}),`
`]}}),`
`,e(t,{get children(){return e(n.p,{get children(){return["For local paths in ",e(n.code,{children:"schema.catalog.paths"}),", relative paths are also resolved from the directory containing the loaded config file."]}})}}),`
`,e(n.h3,{id:"schemas",className:"group relative heading-with-anchor",children:"schemas"}),`
`,e(n.p,{children:"Define custom schemas for specific TOML files. You can specify both root schemas and sub-schemas."}),`
`,e(n.h3,{id:"root-schema",className:"group relative heading-with-anchor",children:"Root Schema"}),`
`,e(n.p,{children:"Apply a schema to the entire TOML document."}),`
`,e(r,{code:`[[schemas]]
toml-version = "v1.0.0"  # optional
path = "https://example.com/schema.json"
include = ["example.toml"]
strict = false

[schemas.lint.rules]
deprecated = "error"`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"toml-version"}),"(optional): The TOML version that the schema is available for"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"path"}),": The schema path (URL or local file path)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"include"}),": File match patterns to apply this schema (supports glob patterns)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"exclude"}),"(optional): File match patterns to skip even if ",e(n.code,{children:"include"})," matched"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"strict"}),"(optional): Override ",e(n.code,{children:"schema.strict"})," for files matched by this root schema"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"lint.rules.deprecated"}),"(optional): Override the deprecated diagnostic level for this schema with ",e(n.code,{children:'"off"'}),", ",e(n.code,{children:'"warn"'}),", or ",e(n.code,{children:'"error"'})]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"format.rules.array-values-order.enabled"}),"(optional): Enable schema-defined array value ordering for this schema"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"format.rules.table-keys-order.enabled"}),"(optional): Enable schema-defined table key ordering for this schema"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"overrides"}),"(optional): Per-target ordering overrides within the matched schema"]}}),`
`]}}),`
`,e(t,{get children(){return[e(n.p,{get children(){return["For local paths, ",e(n.code,{children:"path"})," is resolved relative to the directory containing the loaded config file."]}}),e(n.p,{get children(){return["When using User/System config (for example ",e(n.code,{children:"~/.config/tombi/config.toml"}),`), the config directory is usually outside your project.
In that case, use recursive include globs such as `,e(n.code,{children:"**/project/**/*.toml"})," or filename-based patterns like ",e(n.code,{children:"pyproject.toml"})," to target project files."]}})]}}),`
`,e(n.h3,{id:"sub-schema",className:"group relative heading-with-anchor",children:"Sub Schema"}),`
`,e(n.p,{get children(){return[`Apply a schema to a specific part of the TOML document.
Sub schemas are mainly provided for configuring extension features, such as the `,e(n.a,{href:"https://peps.python.org/pep-0518/#tool-table",children:"tool section"})," in ",e(n.code,{children:"pyproject.toml"})," or similar cases where only part of the document should be validated or structured according to an additional schema."]}}),`
`,e(r,{code:`[[schemas]]
root = "tool.taskipy"
path = "schemas/partial-taskipy.schema.json"
include = ["pyproject.toml"]
strict = false

[schemas.lint.rules]
deprecated = "warn"`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"root"}),": The accessor path to apply the sub-schema (e.g., ",e(n.code,{children:'"tools.tombi"'}),", ",e(n.code,{children:'"tool.*"'}),", ",e(n.code,{children:`"tool.'*'"`}),", ",e(n.code,{children:'"items[1].name"'}),"). Use bare ",e(n.code,{children:"*"})," for any key segment, ",e(n.code,{children:"[*]"})," for any array element, and numeric indices such as ",e(n.code,{children:"[1]"})," for a specific tuple element. If you need the literal key ",e(n.code,{children:"*"}),", quote it as ",e(n.code,{children:"'*'"})," or ",e(n.code,{children:'"*"'}),"."]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"path"}),": The schema path (URL or local file path)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"include"}),": File match patterns to apply this schema (supports glob patterns)"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"exclude"}),"(optional): File match patterns to skip even if ",e(n.code,{children:"include"})," matched"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"strict"}),"(optional): Override the matching root schema or global ",e(n.code,{children:"schema.strict"})," setting below this sub-schema root"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"lint.rules.deprecated"}),"(optional): Override the deprecated diagnostic level for this schema with ",e(n.code,{children:'"off"'}),", ",e(n.code,{children:'"warn"'}),", or ",e(n.code,{children:'"error"'})]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"format.rules.array-values-order.enabled"}),"(optional): Enable schema-defined array value ordering for this schema"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"format.rules.table-keys-order.enabled"}),"(optional): Enable schema-defined table key ordering for this schema"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"overrides"}),"(optional): Per-target ordering overrides within the matched schema"]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-format",className:"group relative heading-with-anchor",children:"schemas[*].format"}),`
`,e(n.p,{children:"Configure schema-specific formatting settings."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-format-rules",className:"group relative heading-with-anchor",children:"schemas[*].format.rules"}),`
`,e(n.p,{children:"Configure schema-specific format rules."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.p,{children:"These rules currently control whether ordering metadata from the matched JSON Schema is applied."}),`
`,e(n.h3,{id:"schemas-format-rules-array-values-order",className:"group relative heading-with-anchor",children:"schemas[*].format.rules.array-values-order"}),`
`,e(n.p,{children:"Configure schema-defined ordering for array values."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.p,{get children(){return["Use ",e(n.code,{children:"{ enabled = false }"})," to ignore schema-defined ordering for array values."]}}),`
`,e(n.h3,{id:"schemas-format-rules-array-values-order-enabled",className:"group relative heading-with-anchor",children:"schemas[*].format.rules.array-values-order.enabled"}),`
`,e(n.p,{children:"Enable or disable schema-defined array value ordering."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-format-rules-table-keys-order",className:"group relative heading-with-anchor",children:"schemas[*].format.rules.table-keys-order"}),`
`,e(n.p,{children:"Configure schema-defined ordering for table keys."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.p,{get children(){return["Use ",e(n.code,{children:"{ enabled = false }"})," to ignore schema-defined ordering for table keys."]}}),`
`,e(n.h3,{id:"schemas-format-rules-table-keys-order-enabled",className:"group relative heading-with-anchor",children:"schemas[*].format.rules.table-keys-order.enabled"}),`
`,e(n.p,{children:"Enable or disable schema-defined table key ordering."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-lint",className:"group relative heading-with-anchor",children:"schemas[*].lint"}),`
`,e(n.p,{children:"Configure schema-specific lint settings."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-lint-rules",className:"group relative heading-with-anchor",children:"schemas[*].lint.rules"}),`
`,e(n.p,{children:"Configure schema-specific lint rules."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-lint-rules-deprecated",className:"group relative heading-with-anchor",children:"schemas[*].lint.rules.deprecated"}),`
`,e(n.p,{get children(){return[`Override the deprecated diagnostic level for the schema.
This applies when the matched JSON Schema marks a value as `,e(n.code,{children:"deprecated: true"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"warn"'})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-overrides",className:"group relative heading-with-anchor",children:"schemas[*].overrides"}),`
`,e(n.p,{children:"Override schema-derived ordering behavior for specific accessor targets within a matched schema."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table[]"})]}}),`
`]}}),`
`,e(r,{code:`[[schemas]]
path = "https://example.com/schema.json"
include = ["example.toml"]

[[schemas.overrides]]
targets = [""]

[schemas.overrides.format.rules]
table-keys-order = "ascending"

[[schemas.overrides]]
targets = ["items[*]", "tool.tasks[*]"]

[schemas.overrides.format.rules]
array-values-order.enabled = false`,language:"toml"}),`
`,e(t,{get children(){return e(n.p,{get children(){return[`If both a root schema override and a sub schema override match the same target, the sub schema override takes precedence.
This precedence is shared by both `,e(n.code,{children:"format.rules.table-keys-order"})," and ",e(n.code,{children:"format.rules.array-values-order"}),"."]}})}}),`
`,e(n.h3,{id:"schemas-overrides-targets",className:"group relative heading-with-anchor",get children(){return["schemas[",e(n.em,{children:"].overrides["}),"].targets"]}}),`
`,e(n.p,{children:"Accessor patterns to which the override applies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"String[]"})]}}),`
`,e(n.li,{get children(){return["Required: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.p,{get children(){return["Use ",e(n.code,{children:'""'})," to target the root table. Use ",e(n.code,{children:"*"})," for any key segment, ",e(n.code,{children:"[*]"})," for any array element, and numeric indices such as ",e(n.code,{children:"[1]"})," for a specific tuple element."]}}),`
`,e(n.h3,{id:"schemas-overrides-format",className:"group relative heading-with-anchor",get children(){return["schemas[",e(n.em,{children:"].overrides["}),"].format"]}}),`
`,e(n.p,{children:"Override schema-specific format settings for matched targets."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-overrides-format-rules",className:"group relative heading-with-anchor",get children(){return["schemas[",e(n.em,{children:"].overrides["}),"].format.rules"]}}),`
`,e(n.p,{children:"Override schema-derived ordering rules for matched targets."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-overrides-format-rules-array-values-order",className:"group relative heading-with-anchor",get children(){return["schemas[",e(n.em,{children:"].overrides["}),"].format.rules.array-values-order"]}}),`
`,e(n.p,{children:"Override array value ordering for matched targets."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'Table | "ascending" | "descending" | "schema" | "version-sort"'})]}}),`
`]}}),`
`,e(n.p,{get children(){return["Set an explicit order to force that ordering, or use ",e(n.code,{children:"{ enabled = false }"})," to disable schema-defined ordering for the matched targets. ",e(n.code,{children:"{ enabled = true }"})," re-enables schema-defined ordering without forcing a specific order."]}}),`
`,e(n.h3,{id:"schemas-overrides-format-rules-table-keys-order",className:"group relative heading-with-anchor",get children(){return["schemas[",e(n.em,{children:"].overrides["}),"].format.rules.table-keys-order"]}}),`
`,e(n.p,{children:"Override table key ordering for matched targets."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'Table | "ascending" | "descending" | "schema" | "version-sort"'})]}}),`
`]}}),`
`,e(n.p,{get children(){return["Set an explicit order to force that ordering. ",e(n.code,{children:'"schema"'})," means to follow the order defined by the JSON Schema. ",e(n.code,{children:"{ enabled = false }"})," disables schema-defined ordering for the matched targets, and ",e(n.code,{children:"{ enabled = true }"})," re-enables it."]}}),`
`,e(n.h3,{id:"schemas-overrides-lint",className:"group relative heading-with-anchor",get children(){return["schemas[",e(n.em,{children:"].overrides["}),"].lint"]}}),`
`,e(n.p,{children:"Override schema-specific lint settings for matched targets."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-overrides-lint-rules",className:"group relative heading-with-anchor",get children(){return["schemas[",e(n.em,{children:"].overrides["}),"].lint.rules"]}}),`
`,e(n.p,{children:"Override schema-specific lint rules for matched targets."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"schemas-overrides-lint-rules-deprecated",className:"group relative heading-with-anchor",get children(){return["schemas[",e(n.em,{children:"].overrides["}),"].lint.rules.deprecated"]}}),`
`,e(n.p,{get children(){return[`Override the deprecated diagnostic level for matched targets.
This applies when the matched JSON Schema marks a value as `,e(n.code,{children:"deprecated: true"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"warn"'})]}}),`
`]}}),`
`,e(n.h3,{id:"overrides",className:"group relative heading-with-anchor",children:"overrides"}),`
`,e(n.p,{children:"Override configuration settings for specific file patterns. You can define multiple override rules, and matching settings will be applied to files."}),`
`,e(r,{code:`[[overrides]]
files.include = ["tests/**/*.toml"]

[overrides.format.rules]
indent-width = 4
line-width = 120

[overrides.lint.rules]
key-empty = "off"`,language:"toml"}),`
`,e(n.h3,{id:"overrides-files",className:"group relative heading-with-anchor",children:"overrides[*].files"}),`
`,e(n.p,{children:"Specify file patterns to which the override should be applied."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`,e(n.li,{get children(){return["Required: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"overrides-files-include",className:"group relative heading-with-anchor",children:"overrides[*].files.include"}),`
`,e(n.p,{children:"File patterns to apply the override. Supports glob patterns."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"String[]"})]}}),`
`,e(n.li,{get children(){return["Required: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"overrides-files-exclude",className:"group relative heading-with-anchor",children:"overrides[*].files.exclude"}),`
`,e(n.p,{children:"File patterns to exclude from the override. Supports glob patterns."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"String[]"})]}}),`
`]}}),`
`,e(n.h3,{id:"overrides-format",className:"group relative heading-with-anchor",children:"overrides[*].format"}),`
`,e(n.p,{children:"Override format settings."}),`
`,e(n.h3,{id:"overrides-format-enabled",className:"group relative heading-with-anchor",children:"overrides[*].format.enabled"}),`
`,e(n.p,{children:"Enable or disable formatting feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"overrides-format-rules",className:"group relative heading-with-anchor",children:"overrides[*].format.rules"}),`
`,e(n.p,{children:"Override format rules. Only the specified rules will be overridden, and the rest will use the base configuration."}),`
`,e(n.p,{get children(){return["Available rules are the same as ",e(n.a,{href:"#format-rules",children:"format.rules"}),"."]}}),`
`,e(n.h3,{id:"overrides-lint",className:"group relative heading-with-anchor",children:"overrides[*].lint"}),`
`,e(n.p,{children:"Override lint settings."}),`
`,e(n.h3,{id:"overrides-lint-enabled",className:"group relative heading-with-anchor",children:"overrides[*].lint.enabled"}),`
`,e(n.p,{children:"Enable or disable linting feature."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"overrides-lint-rules",className:"group relative heading-with-anchor",children:"overrides[*].lint.rules"}),`
`,e(n.p,{children:"Override lint rules. Only the specified rules will be overridden, and the rest will use the base configuration."}),`
`,e(n.p,{get children(){return["Available rules are the same as ",e(n.a,{href:"#lint-rules",children:"lint.rules"}),"."]}}),`
`,e(n.h3,{id:"extensions",className:"group relative heading-with-anchor",children:"extensions"}),`
`,e(n.p,{children:"Configure experimental built-in extensions and their feature flags."}),`
`,e(n.p,{children:"If an extension or feature is omitted, it is enabled by default."}),`
`,e(r,{code:`[extensions]
"tombi-toml/cargo" = { lsp.completion.path.enabled = false }
"tombi-toml/pyproject" = { lsp.document-link.pypi-org.enabled = false }`,language:"toml"}),`
`,e(n.p,{children:"Available extension IDs:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"tombi-toml/cargo"}),": Cargo-specific completion, navigation, document links, code actions, hover, and inlay hints for ",e(n.code,{children:"Cargo.toml"})]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"tombi-toml/pyproject"}),": ",e(n.code,{children:"pyproject.toml"}),"-specific completion, navigation, document links, code actions, hover, and inlay hints"]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"tombi-toml/tombi"}),": Tombi's own config-oriented path completion, navigation, document links, and hover"]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo"'}),`
`,e(n.p,{get children(){return["Configure the experimental Cargo extension for ",e(n.code,{children:"Cargo.toml"}),"."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo",children:"Cargo Extension"})," for the supported Cargo-specific editor features."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".enabled'}),`
`,e(n.p,{children:"Enable or disable the entire Cargo extension."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint'}),`
`,e(n.p,{get children(){return["Configure diagnostics produced for paths declared in ",e(n.code,{children:"Cargo.toml"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo extension lint rules."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint-rules",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint.rules'}),`
`,e(n.p,{get children(){return["Set the severity of individual Cargo extension diagnostics. Each rule defaults to ",e(n.code,{children:'"error"'}),". Available values are ",e(n.code,{children:'"off"'}),", ",e(n.code,{children:'"warn"'}),", and ",e(n.code,{children:'"error"'}),"."]}}),`
`,e(r,{code:`[extensions]
"tombi-toml/cargo" = {
  lint.rules = {
    missing-build-script = "error",
    missing-license-file = "error",
    missing-path-dependency = "error",
    missing-readme = "error",
    missing-target-path = "error",
    missing-workspace = "error",
    missing-workspace-member = "error",
  },
}`,language:"toml"}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint-rules-missing-build-script",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint.rules.missing-build-script'}),`
`,e(n.p,{get children(){return["Report an explicitly configured build script path that does not exist. ",e(n.code,{children:"build = true"})," checks ",e(n.code,{children:"build.rs"}),"; ",e(n.code,{children:"build = false"})," is ignored."]}}),`
`,e(r,{code:`[package]
build = "build/build-missing.rs" # this file does not exist`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint-rules-missing-license-file",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint.rules.missing-license-file'}),`
`,e(n.p,{children:"Report an explicitly configured package or workspace license file path that does not exist."}),`
`,e(r,{code:`[package]
license-file = "LICENSE-missing.txt" # this file does not exist`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint-rules-missing-path-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint.rules.missing-path-dependency'}),`
`,e(n.p,{get children(){return["Report path dependencies whose directory does not contain a ",e(n.code,{children:"Cargo.toml"})," file."]}}),`
`,e(r,{code:`[dependencies]
local-crate = { path = "../local-crate" } # this directory has no Cargo.toml`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint-rules-missing-readme",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint.rules.missing-readme'}),`
`,e(n.p,{get children(){return["Report an explicitly configured package or workspace README path that does not exist. ",e(n.code,{children:"readme = true"})," checks ",e(n.code,{children:"README.md"}),"."]}}),`
`,e(r,{code:`[package]
readme = "README-missing.md" # this file does not exist`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint-rules-missing-target-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint.rules.missing-target-path'}),`
`,e(n.p,{get children(){return["Report missing explicit ",e(n.code,{children:"path"})," values in ",e(n.code,{children:"[lib]"}),", ",e(n.code,{children:"[[bin]]"}),", ",e(n.code,{children:"[[example]]"}),", ",e(n.code,{children:"[[test]]"}),", and ",e(n.code,{children:"[[bench]]"})," targets. Inferred Cargo target paths are not checked."]}}),`
`,e(r,{code:`[[bin]]
name = "tool"
path = "src/bin/tool-missing.rs" # this file does not exist`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint-rules-missing-workspace",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint.rules.missing-workspace'}),`
`,e(n.p,{get children(){return["Report ",e(n.code,{children:"workspace = true"})," inheritance when its matching ",e(n.code,{children:"[workspace.package]"})," field, ",e(n.code,{children:"[workspace.dependencies]"})," entry, or ",e(n.code,{children:"[workspace.lints]"})," table cannot be found."]}}),`
`,e(r,{code:`[dependencies]
shared-crate = { workspace = true } # no matching [workspace.dependencies] entry`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lint-rules-missing-workspace-member",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lint.rules.missing-workspace-member'}),`
`,e(n.p,{get children(){return["Report a ",e(n.code,{children:"workspace.members"})," or ",e(n.code,{children:"workspace.default-members"})," entry whose directory does not contain a ",e(n.code,{children:"Cargo.toml"})," file. Glob patterns are expanded and every matched directory is checked, except directories removed by ",e(n.code,{children:"workspace.exclude"}),". A glob that matches no directory is also reported."]}}),`
`,e(r,{code:`[workspace]
members = ["crates/missing"] # this directory has no Cargo.toml`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp'}),`
`,e(n.p,{children:"Configure Cargo extension language-server features such as completion, navigation, document links, and code actions."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo",children:"Cargo Extension"})," for feature details."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo extension language-server features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-completion",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.completion'}),`
`,e(n.p,{children:"Configure Cargo-specific completion features."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-completion",children:"Cargo Extension > Code Completion"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-completion-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.completion.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo completion features provided by the extension."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-completion-dependency-version",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.completion.dependency-version'}),`
`,e(n.p,{children:`Configure registry version completion for Cargo dependencies.
This powers the version suggestions shown while editing dependency requirements.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-completion",children:"Cargo Extension > Code Completion"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-completion-dependency-version-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.completion.dependency-version.enabled'}),`
`,e(n.p,{children:"Enable or disable version completion for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-completion-dependency-feature",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.completion.dependency-feature'}),`
`,e(n.p,{children:`Configure feature completion for Cargo dependencies.
This powers feature-name suggestions for crate dependencies, including workspace-aware lookups.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-completion",children:"Cargo Extension > Code Completion"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-completion-dependency-feature-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.completion.dependency-feature.enabled'}),`
`,e(n.p,{children:"Enable or disable feature completion for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-completion-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.completion.path'}),`
`,e(n.p,{get children(){return[`Configure path completion for Cargo manifests.
This covers member paths, dependency paths, target source paths, and file references such as `,e(n.code,{children:"readme"}),"."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-completion",children:"Cargo Extension > Code Completion"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-completion-path-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.completion.path.enabled'}),`
`,e(n.p,{children:"Enable or disable path completion for Cargo manifests."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-definition",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-definition'}),`
`,e(n.p,{children:"Configure Cargo-specific go to definition features."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#go-to-definition",children:"Cargo Extension > Go to Definition"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-definition-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-definition.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo go to definition features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-definition-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-definition.dependency'}),`
`,e(n.p,{children:`Configure go to definition for Cargo dependencies.
This resolves dependency entries to workspace definitions or referenced crates when possible.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#go-to-definition",children:"Cargo Extension > Go to Definition"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-definition-dependency-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-definition.dependency.enabled'}),`
`,e(n.p,{children:"Enable or disable go to definition for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-definition-member",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-definition.member'}),`
`,e(n.p,{children:`Configure go to definition for Cargo workspace members.
This resolves workspace member references such as member manifests and managed crates.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#go-to-definition",children:"Cargo Extension > Go to Definition"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-definition-member-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-definition.member.enabled'}),`
`,e(n.p,{children:"Enable or disable go to definition for Cargo workspace members."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-definition-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-definition.path'}),`
`,e(n.p,{get children(){return[`Configure go to definition for Cargo path targets.
This resolves local `,e(n.code,{children:"path"})," references in Cargo manifests."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#go-to-definition",children:"Cargo Extension > Go to Definition"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-definition-path-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-definition.path.enabled'}),`
`,e(n.p,{children:"Enable or disable go to definition for Cargo path targets."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-references",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.references'}),`
`,e(n.p,{children:"Configure Cargo-specific references features."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#references",children:"Cargo Extension > References"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-references-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.references.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo references features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-references-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.references.dependency'}),`
`,e(n.p,{children:`Configure references for Cargo dependencies.
This lists dependency-related usages such as feature activations and workspace dependency consumers.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#references",children:"Cargo Extension > References"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-references-dependency-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.references.dependency.enabled'}),`
`,e(n.p,{children:"Enable or disable references for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-declaration",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-declaration'}),`
`,e(n.p,{children:"Configure Cargo-specific go to declaration features."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#go-to-declaration",children:"Cargo Extension > Go to Declaration"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-declaration-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-declaration.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo go to declaration features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-declaration-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-declaration.dependency'}),`
`,e(n.p,{children:`Configure go to declaration for Cargo dependencies.
This resolves dependency entries back to their declared workspace location when applicable.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#go-to-declaration",children:"Cargo Extension > Go to Declaration"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-declaration-dependency-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-declaration.dependency.enabled'}),`
`,e(n.p,{children:"Enable or disable go to declaration for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-declaration-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-declaration.path'}),`
`,e(n.p,{get children(){return[`Configure go to declaration for Cargo path targets.
This resolves declaration-like navigation for local `,e(n.code,{children:"path"})," references in Cargo manifests."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#go-to-declaration",children:"Cargo Extension > Go to Declaration"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-goto-declaration-path-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.goto-declaration.path.enabled'}),`
`,e(n.p,{children:"Enable or disable go to declaration for Cargo path targets."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-document-link",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.document-link'}),`
`,e(n.p,{children:"Configure Cargo-specific document links."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#document-links",children:"Cargo Extension > Document Links"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-document-link-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.document-link.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo document links."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-document-link-crates-io",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.document-link.crates-io'}),`
`,e(n.p,{children:`Configure document links to crates.io pages referenced from Cargo manifests.
These links open crates.io package pages for registry dependencies.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#document-links",children:"Cargo Extension > Document Links"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-document-link-crates-io-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.document-link.crates-io.enabled'}),`
`,e(n.p,{children:"Enable or disable document links to crates.io pages referenced from Cargo manifests."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action'}),`
`,e(n.p,{children:"Configure Cargo-specific code actions."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-actions",children:"Cargo Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo code actions."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-inherit-from-workspace",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.inherit-from-workspace'}),`
`,e(n.p,{children:`Configure the "Inherit from Workspace" code action for Cargo manifests.
This refactor converts package fields to workspace inheritance.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-actions",children:"Cargo Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-inherit-from-workspace-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.inherit-from-workspace.enabled'}),`
`,e(n.p,{children:'Enable or disable the "Inherit from Workspace" code action for Cargo manifests.'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-add-to-workspace-and-inherit-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.add-to-workspace-and-inherit-dependency'}),`
`,e(n.p,{get children(){return[`Configure the "Add to Workspace and Inherit Dependency" code action for Cargo manifests.
This refactor promotes a member dependency into `,e(n.code,{children:"[workspace.dependencies]"})," and rewrites the member entry to inherit it."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-actions",children:"Cargo Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-add-to-workspace-and-inherit-dependency-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.add-to-workspace-and-inherit-dependency.enabled'}),`
`,e(n.p,{children:'Enable or disable the "Add to Workspace and Inherit Dependency" code action for Cargo manifests.'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-convert-dependency-to-table-format",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.convert-dependency-to-table-format'}),`
`,e(n.p,{children:`Configure the "Convert Dependency to Table Format" code action for Cargo manifests.
This refactor expands a simple string dependency into table form for further editing.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-actions",children:"Cargo Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-convert-dependency-to-table-format-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.convert-dependency-to-table-format.enabled'}),`
`,e(n.p,{children:'Enable or disable the "Convert Dependency to Table Format" code action for Cargo manifests.'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-inherit-dependency-from-workspace",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.inherit-dependency-from-workspace'}),`
`,e(n.p,{children:`Configure the "Inherit Dependency from Workspace" code action for Cargo manifests.
This refactor rewrites dependencies to use workspace-managed versions.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-actions",children:"Cargo Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-inherit-dependency-from-workspace-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.inherit-dependency-from-workspace.enabled'}),`
`,e(n.p,{children:'Enable or disable the "Inherit Dependency from Workspace" code action for Cargo manifests.'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-update-dependency-to-latest-version",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.update-dependency-to-latest-version'}),`
`,e(n.p,{children:`Configure the "Update Dependency to Latest Version" code action for Cargo manifests.
This refactor rewrites dependency versions to the latest published version.`}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#code-actions",children:"Cargo Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-code-action-update-dependency-to-latest-version-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.code-action.update-dependency-to-latest-version.enabled'}),`
`,e(n.p,{children:'Enable or disable the "Update Dependency to Latest Version" code action for Cargo manifests.'}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-hover",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.hover'}),`
`,e(n.p,{children:"Configure Cargo-specific hover features."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#hover",children:"Cargo Extension > Hover"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-hover-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.hover.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo hover features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-hover-dependency-detail",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.hover.dependency-detail'}),`
`,e(n.p,{children:"Configure dependency detail hover for Cargo dependencies."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#hover",children:"Cargo Extension > Hover"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-hover-dependency-detail-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.hover.dependency-detail.enabled'}),`
`,e(n.p,{children:"Enable or disable dependency detail hover for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-hover-feature-dependencies",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.hover.feature-dependencies'}),`
`,e(n.p,{children:"Configure feature dependency hover for Cargo dependencies."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#hover",children:"Cargo Extension > Hover"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-hover-feature-dependencies-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.hover.feature-dependencies.enabled'}),`
`,e(n.p,{children:"Enable or disable feature dependency hover for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-hover-default-features",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.hover.default-features'}),`
`,e(n.p,{children:"Configure default feature hover for Cargo dependencies."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#hover",children:"Cargo Extension > Hover"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-hover-default-features-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.hover.default-features.enabled'}),`
`,e(n.p,{children:"Enable or disable default feature hover for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-inlay-hint",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.inlay-hint'}),`
`,e(n.p,{children:"Configure Cargo-specific inlay hints."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#inlay-hints",children:"Cargo Extension > Inlay Hints"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-inlay-hint-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.inlay-hint.enabled'}),`
`,e(n.p,{children:"Enable or disable all Cargo inlay hints."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-inlay-hint-dependency-version",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.inlay-hint.dependency-version'}),`
`,e(n.p,{children:"Configure resolved-version inlay hints for Cargo dependencies."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#inlay-hints",children:"Cargo Extension > Inlay Hints"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-inlay-hint-dependency-version-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.inlay-hint.dependency-version.enabled'}),`
`,e(n.p,{children:"Enable or disable resolved-version inlay hints for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-inlay-hint-default-features",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.inlay-hint.default-features'}),`
`,e(n.p,{children:"Configure default-features inlay hints for Cargo dependencies."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#inlay-hints",children:"Cargo Extension > Inlay Hints"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-inlay-hint-default-features-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.inlay-hint.default-features.enabled'}),`
`,e(n.p,{children:"Enable or disable default-features inlay hints for Cargo dependencies."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-inlay-hint-workspace-value",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.inlay-hint.workspace-value'}),`
`,e(n.p,{children:"Configure workspace-value inlay hints for Cargo workspace inheritance."}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-cargo#inlay-hints",children:"Cargo Extension > Inlay Hints"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-cargo-lsp-inlay-hint-workspace-value-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/cargo".lsp.inlay-hint.workspace-value.enabled'}),`
`,e(n.p,{children:"Enable or disable workspace-value inlay hints for Cargo workspace inheritance."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject"'}),`
`,e(n.p,{get children(){return["Configure the experimental ",e(n.code,{children:"pyproject.toml"})," extension."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject",children:"Pyproject Extension"})," for the supported ",e(n.code,{children:"pyproject.toml"})," editor features."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".enabled'}),`
`,e(n.p,{get children(){return["Enable or disable the entire ",e(n.code,{children:"pyproject.toml"})," extension."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint'}),`
`,e(n.p,{get children(){return["Configure the severity of diagnostics produced by the ",e(n.code,{children:"pyproject.toml"})," extension."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules'}),`
`,e(n.p,{get children(){return["Set the severity for individual pyproject diagnostics. Each rule defaults to its current severity when omitted. Available values are ",e(n.code,{children:'"off"'}),", ",e(n.code,{children:'"warn"'}),", and ",e(n.code,{children:'"error"'}),"."]}}),`
`,e(r,{code:`[extensions]
"tombi-toml/pyproject" = {
  lint.rules = {
    ambiguous-import-name = "error",
    cyclic-include-group = "error",
    duplicate-dependency-group = "error",
    duplicate-extra-name = "error",
    duplicate-import-name = "warn",
    invalid-dependency-specifier = "error",
    invalid-email-display-name = "error",
    invalid-entry-point-target = "error",
    invalid-import-name = "error",
    missing-import-name-parent = "warn",
    missing-workspace-member = "error",
    undefined-include-group = "error",
    unknown-classifier = "warn",
  },
}`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-ambiguous-import-name",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.ambiguous-import-name'}),`
`,e(n.p,{children:"Set the severity for names declared as both exclusive and shared imports."}),`
`,e(r,{code:`[project]
import-names = ["acme"] # conflicts with the same name declared below
import-namespaces = ["acme"] # conflicts with the same name declared above`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-cyclic-include-group",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.cyclic-include-group'}),`
`,e(n.p,{children:"Report a cycle in dependency group includes."}),`
`,e(r,{code:`[dependency-groups]
a = [{ include-group = "b" }] # includes b
b = [{ include-group = "a" }] # includes a, creating a cycle`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-duplicate-dependency-group",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.duplicate-dependency-group'}),`
`,e(n.p,{get children(){return["Report dependency group names that become identical after lowercasing and replacing runs of ",e(n.code,{children:"-"}),", ",e(n.code,{children:"_"}),", or ",e(n.code,{children:"."})," with ",e(n.code,{children:"-"}),"."]}}),`
`,e(r,{code:`[dependency-groups]
qa_tools = [] # normalizes to "qa-tools", same as the name below
"QA.Tools" = [] # normalizes to "qa-tools", same as the name above`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-duplicate-extra-name",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.duplicate-extra-name'}),`
`,e(n.p,{get children(){return["Report optional dependency extra names that become identical after lowercasing and replacing runs of ",e(n.code,{children:"-"}),", ",e(n.code,{children:"_"}),", or ",e(n.code,{children:"."})," with ",e(n.code,{children:"-"}),"."]}}),`
`,e(r,{code:`[project.optional-dependencies]
qa_tools = [] # normalizes to "qa-tools", same as the name below
"QA.Tools" = [] # normalizes to "qa-tools", same as the name above`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-duplicate-import-name",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.duplicate-import-name'}),`
`,e(n.p,{children:"Set the severity for duplicate Python import names."}),`
`,e(r,{code:`[project]
import-names = ["acme", "acme"] # the same import name is declared twice`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"warn"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-invalid-dependency-specifier",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.invalid-dependency-specifier'}),`
`,e(n.p,{children:"Set the severity for dependency strings that do not follow PEP 508."}),`
`,e(r,{code:`[project]
dependencies = ["not a valid requirement !!!"] # not a valid PEP 508 requirement`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-invalid-email-display-name",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.invalid-email-display-name'}),`
`,e(n.p,{children:"Set the severity for invalid author and maintainer email display names."}),`
`,e(r,{code:`[project]
authors = [{ name = "Doe, Jane" }] # commas are not allowed in the display name`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-invalid-entry-point-target",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.invalid-entry-point-target'}),`
`,e(n.p,{children:"Set the severity for invalid Python entry point targets."}),`
`,e(r,{code:'[project.scripts]\ncli = "my_package.class" # `class` is a Python keyword and cannot be an attribute',language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-invalid-import-name",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.invalid-import-name'}),`
`,e(n.p,{children:"Set the severity for invalid Python import names."}),`
`,e(r,{code:'[project]\nimport-names = ["if"] # `if` is a Python keyword, not a valid import name',language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-missing-import-name-parent",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.missing-import-name-parent'}),`
`,e(n.p,{children:"Set the severity for import names whose parent is not declared."}),`
`,e(r,{code:'[project]\nimport-namespaces = ["acme.plugins"] # parent `acme` is not declared in either list',language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"warn"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-missing-workspace-member",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.missing-workspace-member'}),`
`,e(n.p,{get children(){return["Set the severity for a non-glob ",e(n.code,{children:"tool.uv.workspace.members"})," path whose directory does not contain a ",e(n.code,{children:"pyproject.toml"})," file. Glob patterns are not checked, because uv silently ignores member directories without a ",e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(r,{code:`[tool.uv.workspace]
members = ["packages/missing"] # this directory has no pyproject.toml`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-undefined-include-group",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.undefined-include-group'}),`
`,e(n.p,{get children(){return["Report a dependency group that includes a group not defined in ",e(n.code,{children:"[dependency-groups]"}),"."]}}),`
`,e(r,{code:`[dependency-groups]
dev = [{ include-group = "test" }] # no [dependency-groups].test exists`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lint-rules-unknown-classifier",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lint.rules.unknown-classifier'}),`
`,e(n.p,{children:"Set the severity for classifiers not listed in the Trove classifier specification."}),`
`,e(r,{code:`[project]
classifiers = ["NotAClassifier :: DefinitelyInvalid"] # not in the Trove classifier list`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"warn"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp'}),`
`,e(n.p,{get children(){return["Configure ",e(n.code,{children:"pyproject.toml"})," extension language-server features such as completion, navigation, document links, and code actions."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject",children:"Pyproject Extension"})," for feature details."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable all ",e(n.code,{children:"pyproject.toml"})," extension language-server features."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-completion",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.completion'}),`
`,e(n.p,{get children(){return["Configure ",e(n.code,{children:"pyproject.toml"}),"-specific completion features."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#completion",children:"Pyproject Extension > Completion"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-completion-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.completion.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable all ",e(n.code,{children:"pyproject.toml"})," completion features provided by the extension."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-completion-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.completion.path'}),`
`,e(n.p,{get children(){return["Configure path completion for ",e(n.code,{children:"pyproject.toml"}),`.
This covers workspace member paths, source paths, and file references such as `,e(n.code,{children:"readme"})," and ",e(n.code,{children:"license.file"}),"."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#completion",children:"Pyproject Extension > Completion"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-completion-path-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.completion.path.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable path completion for ",e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-definition",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-definition'}),`
`,e(n.p,{get children(){return["Configure ",e(n.code,{children:"pyproject.toml"}),"-specific go to definition features."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#go-to-definition",children:"Pyproject Extension > Go to Definition"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-definition-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-definition.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable all ",e(n.code,{children:"pyproject.toml"})," go to definition features."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-definition-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-definition.dependency'}),`
`,e(n.p,{get children(){return["Configure go to definition for ",e(n.code,{children:"pyproject.toml"}),` dependencies.
This resolves dependency strings or source entries to workspace-managed packages when possible.`]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#go-to-definition",children:"Pyproject Extension > Go to Definition"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-definition-dependency-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-definition.dependency.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable go to definition for ",e(n.code,{children:"pyproject.toml"})," dependencies."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-definition-member",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-definition.member'}),`
`,e(n.p,{get children(){return["Configure go to definition for ",e(n.code,{children:"pyproject.toml"}),` workspace members.
This resolves uv workspace member references to the corresponding member project.`]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#go-to-definition",children:"Pyproject Extension > Go to Definition"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-definition-member-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-definition.member.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable go to definition for ",e(n.code,{children:"pyproject.toml"})," workspace members."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-definition-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-definition.path'}),`
`,e(n.p,{get children(){return["Configure go to definition for ",e(n.code,{children:"pyproject.toml"}),` path targets.
This resolves local `,e(n.code,{children:"path"})," references in ",e(n.code,{children:"tool.uv.sources"})," and related fields."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#go-to-definition",children:"Pyproject Extension > Go to Definition"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-definition-path-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-definition.path.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable go to definition for ",e(n.code,{children:"pyproject.toml"})," path targets."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-references",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.references'}),`
`,e(n.p,{get children(){return["Configure ",e(n.code,{children:"pyproject.toml"}),"-specific references features."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#references",children:"Pyproject Extension > References"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-references-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.references.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable all ",e(n.code,{children:"pyproject.toml"})," references features."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-references-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.references.dependency'}),`
`,e(n.p,{get children(){return["Configure references for ",e(n.code,{children:"pyproject.toml"}),` dependencies.
This lists workspace-backed dependency usages and related include-group consumers.`]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#references",children:"Pyproject Extension > References"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-references-dependency-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.references.dependency.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable references for ",e(n.code,{children:"pyproject.toml"})," dependencies."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-declaration",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-declaration'}),`
`,e(n.p,{get children(){return["Configure ",e(n.code,{children:"pyproject.toml"}),"-specific go to declaration features."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#go-to-declaration",children:"Pyproject Extension > Go to Declaration"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-declaration-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-declaration.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable all ",e(n.code,{children:"pyproject.toml"})," go to declaration features."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-declaration-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-declaration.dependency'}),`
`,e(n.p,{get children(){return["Configure go to declaration for ",e(n.code,{children:"pyproject.toml"}),` dependencies.
This resolves dependency references back to their workspace declaration.`]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#go-to-declaration",children:"Pyproject Extension > Go to Declaration"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-declaration-dependency-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-declaration.dependency.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable go to declaration for ",e(n.code,{children:"pyproject.toml"})," dependencies."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-declaration-member",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-declaration.member'}),`
`,e(n.p,{get children(){return["Configure go to declaration for ",e(n.code,{children:"pyproject.toml"}),` workspace members.
This resolves uv workspace member references back to the workspace manifest that declares them.`]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#go-to-declaration",children:"Pyproject Extension > Go to Declaration"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-declaration-member-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-declaration.member.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable go to declaration for ",e(n.code,{children:"pyproject.toml"})," workspace members."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-declaration-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-declaration.path'}),`
`,e(n.p,{get children(){return["Configure go to declaration for ",e(n.code,{children:"pyproject.toml"}),` path targets.
This resolves declaration-like navigation for local path references in `,e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#go-to-declaration",children:"Pyproject Extension > Go to Declaration"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-goto-declaration-path-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.goto-declaration.path.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable go to declaration for ",e(n.code,{children:"pyproject.toml"})," path targets."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-document-link",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.document-link'}),`
`,e(n.p,{get children(){return["Configure ",e(n.code,{children:"pyproject.toml"}),"-specific document links."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#document-links",children:"Pyproject Extension > Document Links"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-document-link-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.document-link.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable all ",e(n.code,{children:"pyproject.toml"})," document links."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-document-link-pypi-org",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.document-link.pypi-org'}),`
`,e(n.p,{get children(){return["Configure document links to PyPI pages referenced from ",e(n.code,{children:"pyproject.toml"}),`.
These links open PyPI package pages for dependency strings when no local workspace target applies.`]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#document-links",children:"Pyproject Extension > Document Links"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-document-link-pypi-org-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.document-link.pypi-org.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable document links to PyPI pages referenced from ",e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-code-action",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.code-action'}),`
`,e(n.p,{get children(){return["Configure ",e(n.code,{children:"pyproject.toml"}),"-specific code actions."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#code-actions",children:"Pyproject Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-code-action-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.code-action.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable all ",e(n.code,{children:"pyproject.toml"})," code actions."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-code-action-add-to-workspace-and-use-workspace-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.code-action.add-to-workspace-and-use-workspace-dependency'}),`
`,e(n.p,{get children(){return['Configure the "Add to Workspace and Use Workspace Dependency" code action for ',e(n.code,{children:"pyproject.toml"}),`.
This refactor adds the dependency to the workspace and rewrites the member entry to use the shared definition.`]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#code-actions",children:"Pyproject Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-code-action-add-to-workspace-and-use-workspace-dependency-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.code-action.add-to-workspace-and-use-workspace-dependency.enabled'}),`
`,e(n.p,{get children(){return['Enable or disable the "Add to Workspace and Use Workspace Dependency" code action for ',e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-code-action-update-dependency-to-latest-version",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.code-action.update-dependency-to-latest-version'}),`
`,e(n.p,{get children(){return['Configure the "Update Dependency to Latest Version" code action for ',e(n.code,{children:"pyproject.toml"}),`.
This refactor rewrites dependency versions to the latest published version.`]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#code-actions",children:"Pyproject Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-code-action-update-dependency-to-latest-version-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.code-action.update-dependency-to-latest-version.enabled'}),`
`,e(n.p,{get children(){return['Enable or disable the "Update Dependency to Latest Version" code action for ',e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-code-action-use-workspace-dependency",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.code-action.use-workspace-dependency'}),`
`,e(n.p,{get children(){return['Configure the "Use Workspace Dependency" code action for ',e(n.code,{children:"pyproject.toml"}),`.
This refactor rewrites a member dependency so it follows the workspace-managed dependency definition.`]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#code-actions",children:"Pyproject Extension > Code Actions"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-code-action-use-workspace-dependency-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.code-action.use-workspace-dependency.enabled'}),`
`,e(n.p,{get children(){return['Enable or disable the "Use Workspace Dependency" code action for ',e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-hover",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.hover'}),`
`,e(n.p,{get children(){return["Configure ",e(n.code,{children:"pyproject.toml"}),"-specific hover features."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#hover",children:"Pyproject Extension > Hover"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-hover-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.hover.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable all ",e(n.code,{children:"pyproject.toml"})," hover features provided by the extension."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-hover-dependency-detail",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.hover.dependency-detail'}),`
`,e(n.p,{get children(){return["Configure dependency detail hover for ",e(n.code,{children:"pyproject.toml"})," dependencies."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#hover",children:"Pyproject Extension > Hover"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-hover-dependency-detail-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.hover.dependency-detail.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable dependency detail hover for ",e(n.code,{children:"pyproject.toml"})," dependencies."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-inlay-hint",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.inlay-hint'}),`
`,e(n.p,{get children(){return["Configure ",e(n.code,{children:"pyproject.toml"}),"-specific inlay hints."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#inlay-hints",children:"Pyproject Extension > Inlay Hints"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-inlay-hint-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.inlay-hint.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable all ",e(n.code,{children:"pyproject.toml"})," inlay hints."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-inlay-hint-dependency-version",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.inlay-hint.dependency-version'}),`
`,e(n.p,{get children(){return["Configure resolved-version inlay hints for ",e(n.code,{children:"pyproject.toml"})," dependencies."]}}),`
`,e(n.p,{get children(){return["See ",e(n.a,{href:"/tombi/main/docs/extensions/tombi-extension-pyproject#inlay-hints",children:"Pyproject Extension > Inlay Hints"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-pyproject-lsp-inlay-hint-dependency-version-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/pyproject".lsp.inlay-hint.dependency-version.enabled'}),`
`,e(n.p,{get children(){return["Enable or disable resolved-version inlay hints for ",e(n.code,{children:"pyproject.toml"})," dependencies."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi"'}),`
`,e(n.p,{get children(){return["Configure the experimental Tombi configuration extension used while editing ",e(n.code,{children:"tombi.toml"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".enabled'}),`
`,e(n.p,{children:"Enable or disable the entire Tombi configuration extension."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lint",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lint'}),`
`,e(n.p,{children:"Configure diagnostics produced for local schema and schema catalog paths in Tombi configuration files."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lint-rules",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lint.rules'}),`
`,e(n.p,{get children(){return["Set the severity for individual Tombi configuration diagnostics. Each rule defaults to ",e(n.code,{children:'"error"'})," when omitted. Available values are ",e(n.code,{children:'"off"'}),", ",e(n.code,{children:'"warn"'}),", and ",e(n.code,{children:'"error"'}),"."]}}),`
`,e(r,{code:`[extensions]
"tombi-toml/tombi" = {
  lint.rules = {
    missing-catalog-path = "error",
    missing-extends-path = "error",
    missing-schema-path = "error",
  },
}`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lint-rules-missing-catalog-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lint.rules.missing-catalog-path'}),`
`,e(n.p,{get children(){return["Set the severity for schema catalogs that do not exist: a local path that does not point to a file, a ",e(n.code,{children:"tombi://"})," URL that names no built-in file, or an ",e(n.code,{children:"http(s)://"})," URL for which the server answers 404 or 410 (at most a warning, as it depends on the network). Network failures and offline mode are not reported."]}}),`
`,e(r,{code:`[schema.catalog]
paths = ["schemas/catalog.json"] # no file exists at this path`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lint-rules-missing-extends-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lint.rules.missing-extends-path'}),`
`,e(n.p,{get children(){return["Set the severity for ",e(n.code,{children:"extends"})," entries that do not exist: a local path that does not point to a file, or an ",e(n.code,{children:"http(s)://"})," URL for which the server answers 404 or 410 (at most a warning, as it depends on the network). Network failures and offline mode are not reported."]}}),`
`,e(r,{code:'extends = ["base/tombi.toml"] # no file exists at this path',language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lint-rules-missing-schema-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lint.rules.missing-schema-path'}),`
`,e(n.p,{get children(){return["Set the severity for schemas that do not exist: a local path that does not point to a file, a ",e(n.code,{children:"tombi://"})," URL that names no built-in file, or an ",e(n.code,{children:"http(s)://"})," URL for which the server answers 404 or 410 (at most a warning, as it depends on the network). Network failures and offline mode are not reported."]}}),`
`,e(r,{code:`[[schemas]]
path = "schemas/example.json" # no file exists at this path`,language:"toml"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:'"off" | "warn" | "error"'})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:'"error"'})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp'}),`
`,e(n.p,{children:"Configure Tombi configuration language-server features such as path completion, path navigation, and path document links."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.enabled'}),`
`,e(n.p,{children:"Enable or disable all Tombi configuration language-server features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-completion",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.completion'}),`
`,e(n.p,{children:"Configure Tombi configuration completion features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-completion-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.completion.enabled'}),`
`,e(n.p,{children:"Enable or disable all Tombi configuration completion features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-completion-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.completion.path'}),`
`,e(n.p,{children:`Configure path completion for Tombi configuration files.
This covers local file and directory references used in Tombi configuration.`}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-completion-path-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.completion.path.enabled'}),`
`,e(n.p,{children:"Enable or disable path completion for Tombi configuration files."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-goto-definition",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.goto-definition'}),`
`,e(n.p,{children:"Configure Tombi configuration go to definition features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-goto-definition-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.goto-definition.enabled'}),`
`,e(n.p,{children:"Enable or disable all Tombi configuration go to definition features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-goto-definition-path",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.goto-definition.path'}),`
`,e(n.p,{get children(){return[`Configure go to definition for paths referenced from Tombi configuration files.
This resolves local paths referenced from `,e(n.code,{children:"tombi.toml"}),"."]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-goto-definition-path-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.goto-definition.path.enabled'}),`
`,e(n.p,{children:"Enable or disable go to definition for paths referenced from Tombi configuration files."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-document-link",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.document-link'}),`
`,e(n.p,{children:"Configure Tombi configuration document links."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-document-link-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.document-link.enabled'}),`
`,e(n.p,{children:"Enable or disable all Tombi configuration document links."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-hover",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.hover'}),`
`,e(n.p,{children:"Configure Tombi configuration hover features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Table"})]}}),`
`]}}),`
`,e(n.h3,{id:"extensions-tombi-toml-tombi-lsp-hover-enabled",className:"group relative heading-with-anchor",children:'extensions."tombi-toml/tombi".lsp.hover.enabled'}),`
`,e(n.p,{children:"Enable or disable all Tombi configuration hover features."}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Type: ",e(n.code,{children:"Boolean"})]}}),`
`,e(n.li,{get children(){return["Default: ",e(n.code,{children:"true"})]}}),`
`]}})]}function g(i={}){const{wrapper:n}={...d(),...i.components};return n?e(n,h(i,{get children(){return e(c,i)}})):c(i)}function a(i,n){throw new Error("Expected component `"+i+"` to be defined: you likely forgot to import, pass, or provide it.")}export{g as default};
