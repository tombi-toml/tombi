import{b as e,k as g,i as r,t as o}from"./web-DCz9_uPG.js";import{M as p}from"./index-6OQr06v0.js";import{N as l}from"./Highlight-CMhf5ZLw.js";import{I as i}from"./ImageTabs-ByVWvQJA.js";var u=o('<img src=/tombi/common/v1/Cargo_crate_version_completion.png alt="Crate Version Completion"style="display:block;margin:1rem auto;width:80%">'),m=o('<img src=/tombi/common/v1/Cargo_crate_features_completion.png alt="Crate Feature Completion"style="display:block;margin:1rem auto;width:80%">'),t=o("<br>");function s(c){const n={a:"a",code:"code",h1:"h1",h2:"h2",h3:"h3",h4:"h4",li:"li",p:"p",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",ul:"ul",...p(),...c.components},{CodeBlock:d,PageHeading:a}=n;return d||h("CodeBlock"),a||h("PageHeading"),[e(a,{title:"Cargo Extension",description:"Rust Cargo.toml specific features and validations.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/extensions/tombi-extension-cargo"}),`
`,`
`,e(n.h1,{id:"cargo-extension",className:"group relative heading-with-anchor",children:"Cargo Extension"}),`
`,e(n.p,{children:"Tombi itself is written in Rust, and we want to make the Rust development experience even better 🦀"}),`
`,e(n.p,{get children(){return["Experimentally, we provide additional features when editing ",e(n.code,{children:"Cargo.toml"})," files."]}}),`
`,e(n.h2,{id:"supported-features",className:"group relative heading-with-anchor",children:"Supported Features"}),`
`,e(n.h3,{id:"code-completion",className:"group relative heading-with-anchor",children:"Code Completion"}),`
`,e(n.p,{get children(){return["The extension provides intelligent code completion for ",e(n.code,{children:"Cargo.toml"})," files."]}}),`
`,e(n.h4,{id:"workspace-aware-dependency-completion",className:"group relative heading-with-anchor",children:"Workspace-Aware Dependency Completion"}),`
`,e(n.p,{get children(){return["When a member crate belongs to a Cargo workspace, completion can suggest inheriting dependencies directly from ",e(n.code,{children:"[workspace.dependencies]"}),"."]}}),`
`,e(d,{code:"[dependencies]\nser # <- completion can insert `serde.workspace = true`",language:"toml"}),`
`,e(n.p,{get children(){return["This also works in target-specific dependency tables such as ",e(n.code,{children:"[target.'cfg(unix)'.dependencies]"}),"."]}}),`
`,e(n.h4,{id:"crate-version-completion",className:"group relative heading-with-anchor",children:"Crate Version Completion"}),`
`,e(n.p,{children:"Get version suggestions when specifying crate versions:"}),`
`,e(d,{code:`[dependencies]
serde = "1.0.130"  # <- version suggestions appear here
tokio = { version = "1.21.0" }  # <- also here`,language:"toml"}),`
`,e(n.p,{children:"Features:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{children:"Fetches available versions from crates.io"}),`
`,e(n.li,{children:"Shows up to 100 versions sorted newest first"}),`
`,e(n.li,{get children(){return["Works in all dependency sections (",e(n.code,{children:"dependencies"}),", ",e(n.code,{children:"dev-dependencies"}),", ",e(n.code,{children:"build-dependencies"}),", ",e(n.code,{children:"workspace.dependencies"}),")"]}}),`
`]}}),`
`,r(u),`
`,e(n.h4,{id:"crate-feature-completion",className:"group relative heading-with-anchor",children:"Crate Feature Completion"}),`
`,e(n.p,{children:"Get feature suggestions when specifying crate features:"}),`
`,e(d,{code:`[dependencies]
serde = { version = "1.0", features = ["derive"] }  # <- feature suggestions
tokio = { version = "1.21", features = ["full", "net"] }  # <- in array`,language:"toml"}),`
`,e(n.p,{children:"Features:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Fetches available features from:",`
`,e(n.ul,{get children(){return[`
`,e(n.li,{children:"crates.io for registry dependencies"}),`
`,e(n.li,{get children(){return["Local ",e(n.code,{children:"Cargo.toml"})," for path dependencies"]}}),`
`,e(n.li,{get children(){return["Workspace ",e(n.code,{children:"Cargo.toml"})," for workspace dependencies"]}}),`
`]}}),`
`]}}),`
`,e(n.li,{children:"Shows feature dependencies in documentation"}),`
`,e(n.li,{children:"Filters out already selected features"}),`
`,e(n.li,{get children(){return[e(n.code,{children:"default"})," feature shown first, features starting with ",e(n.code,{children:"_"})," shown last"]}}),`
`]}}),`
`,e(l,{get children(){return e(n.p,{get children(){return["The extension is context-aware: when a dependency uses ",e(n.code,{children:"workspace = true"}),", it looks up features from the workspace definition."]}})}}),`
`,r(m),`
`,e(n.h4,{id:"path-completion",className:"group relative heading-with-anchor",children:"Path Completion"}),`
`,e(n.p,{children:"The extension also provides file and directory completion for common Cargo path fields, including:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"[workspace].members"}),", ",e(n.code,{children:"[workspace].exclude"}),", and ",e(n.code,{children:"[workspace].default-members"})]}}),`
`,e(n.li,{get children(){return["dependency ",e(n.code,{children:"path"})," fields"]}}),`
`,e(n.li,{get children(){return["target paths such as ",e(n.code,{children:"[lib].path"})," and ",e(n.code,{children:"[[bin]].path"})]}}),`
`,e(n.li,{get children(){return["file references such as ",e(n.code,{children:"readme"}),", ",e(n.code,{children:"license-file"}),", ",e(n.code,{children:"include"}),", and ",e(n.code,{children:"exclude"})]}}),`
`]}}),`
`,e(n.h3,{id:"hover",className:"group relative heading-with-anchor",children:"Hover"}),`
`,e(n.p,{children:"For dependency entries, Tombi can enrich hover content with crate metadata."}),`
`,e(d,{code:`[dependencies]
serde = "1.0" # <- crate name and description from crates.io
tombi-extension-cargo = { workspace = true } # <- metadata from the local workspace crate`,language:"toml"}),`
`,e(n.p,{children:"Behavior:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["workspace and local path dependencies show metadata from the referenced crate's ",e(n.code,{children:"Cargo.toml"})]}}),`
`,e(n.li,{children:"registry dependencies can show crate metadata fetched from crates.io"}),`
`,e(n.li,{children:"in offline mode, remote crates.io lookups are skipped and only local workspace/path metadata is used"}),`
`]}}),`
`,e(n.h3,{id:"inlay-hints",className:"group relative heading-with-anchor",children:"Inlay Hints"}),`
`,e(n.p,{get children(){return["The extension can show inline hints for workspace-aware dependency values in ",e(n.code,{children:"Cargo.toml"}),"."]}}),`
`,e(n.p,{get children(){return["Current hints cover resolved dependency versions, ",e(n.code,{children:"default-features"}),", and workspace-inherited values so you can inspect the effective dependency state without leaving the current ",e(n.code,{children:"Cargo.toml"}),"."]}}),`
`,e(n.h3,{id:"lint",className:"group relative heading-with-anchor",children:"Lint"}),`
`,e(n.p,{get children(){return["The Cargo extension checks explicitly configured README, license file, build script, target source, local dependency paths, ",e(n.code,{children:"workspace.members"})," and ",e(n.code,{children:"workspace.default-members"})," paths, and ",e(n.code,{children:"workspace = true"})," inheritance for package metadata, dependencies, and lints. Missing workspace definitions are errors by default. Glob member patterns are expanded and every matched directory is checked, except directories removed by ",e(n.code,{children:"workspace.exclude"}),". Paths inferred by Cargo when a field is omitted are not checked."]}}),`
`,e(n.p,{get children(){return["Configure each rule under ",e(n.code,{children:'[extensions."tombi-toml/cargo".lint.rules]'}),":"]}}),`
`,e(d,{code:`[extensions."tombi-toml/cargo".lint.rules]
missing-build-script = "error"
missing-license-file = "error"
missing-path-dependency = "error"
missing-readme = "error"
missing-target-path = "error"
missing-workspace = "error"
missing-workspace-member = "error"`,language:"toml"}),`
`,e(n.p,{get children(){return["Set a rule to ",e(n.code,{children:'"off"'}),", ",e(n.code,{children:'"warn"'}),", or ",e(n.code,{children:'"error"'}),". See ",e(n.a,{href:"/tombi/common/v1/docs/configuration#extensions-tombi-toml-cargo-lint",children:"Cargo extension configuration"})," for details."]}}),`
`,e(n.h3,{id:"go-to-definition",className:"group relative heading-with-anchor",children:"Go to Definition"}),`
`,e(n.p,{get children(){return["For example, suppose you have a ",e(n.code,{children:"Cargo.toml"})," like the one below:"]}}),`
`,e(n.h4,{id:"go-to-workspace-crate-definition-from-dependencies",className:"group relative heading-with-anchor",get children(){return["Go to workspace/crate definition from ",e(n.code,{children:"dependencies"})]}}),`
`,e(d,{code:`[dependencies]
serde = { workspace = true }
tombi-ast-syntax = { workspace = true }`,language:"toml"}),`
`,e(n.p,{get children(){return["In this case, when your cursor is on ",e(n.code,{children:"workspace"}),`, executing "Go to Definition" will navigate you to the corresponding entry in the workspace's `,e(n.code,{children:"Cargo.toml"}),"."]}}),`
`,e(n.p,{children:"When your cursor is on the dependency key itself, Tombi first checks whether the workspace-managed dependency resolves to a local crate:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return[e(n.code,{children:"serde"})," has no local ",e(n.code,{children:"Cargo.toml"}),', so "Go to Definition" jumps to ',e(n.code,{children:"workspace.dependencies.serde"})," in the workspace ",e(n.code,{children:"Cargo.toml"}),"."]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"tombi-ast-syntax"}),` resolves to a local workspace crate, so "Go to Definition" jumps to that crate's `,e(n.code,{children:"Cargo.toml"})," and lands on the ",e(n.code,{children:"package.name"})," value."]}}),`
`]}}),`
`,e(l,{get children(){return e(n.p,{get children(){return['"Go to Declaration" always uses the matching entry in the workspace ',e(n.code,{children:"Cargo.toml"})," for ",e(n.code,{children:"workspace = true"}),"."]}})}}),`
`,e(n.h4,{id:"go-to-crate-definition-from-workspace-dependencies",className:"group relative heading-with-anchor",get children(){return["Go to crate definition from ",e(n.code,{children:"workspace.dependencies"})]}}),`
`,e(d,{code:`[workspace.dependencies]
tombi-ast-syntax = { path = "crates/tombi-ast-syntax" }`,language:"toml"}),`
`,e(n.p,{get children(){return[`In this case, when your cursor is on the dependency key, executing "Go to Definition" navigates to the target crate's `,e(n.code,{children:"Cargo.toml"})," and lands on its ",e(n.code,{children:"package.name"}),` value when the workspace dependency resolves to a member or local path.
When your cursor is on `,e(n.code,{children:"path"}),', executing "Go to Definition" will navigate you to the crate definition.']}}),`
`,e(n.h3,{id:"go-to-declaration",className:"group relative heading-with-anchor",children:"Go to Declaration"}),`
`,e(d,{code:`[dependencies]
serde = { workspace = true }
tombi-ast-syntax = { workspace = true }`,language:"toml"}),`
`,e(n.p,{get children(){return["In this case, when your cursor is on ",e(n.code,{children:"workspace"}),`, executing "Go to Declaration" will navigate you to the corresponding entry in the workspace's `,e(n.code,{children:"Cargo.toml"}),"."]}}),`
`,e(n.h3,{id:"navigation-list",className:"group relative heading-with-anchor",children:"Navigation List"}),`
`,e(n.p,{get children(){return["The table below describes the intended accessor-by-accessor split between ",e(n.code,{children:"Go to Definition"}),", ",e(n.code,{children:"Go to Declaration"}),", and ",e(n.code,{children:"Find References"}),"."]}}),`
`,e(n.table,{get children(){return[e(n.thead,{get children(){return e(n.tr,{get children(){return[e(n.th,{children:"type"}),e(n.th,{children:"Accessors"}),e(n.th,{children:"Go to Definition"}),e(n.th,{children:"Go to Declaration"}),e(n.th,{children:"Find References"})]}})}}),e(n.tbody,{get children(){return[e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return e(n.code,{children:"package.name"})}}),e(n.td,{get children(){return["Current ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"package.name"})]}}),e(n.td,{children:"-"}),e(n.td,{get children(){return["Usages of this package from member ",e(n.code,{children:"Cargo.toml"})," files and workspace dependency entries"]}})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return e(n.code,{children:"features.*"})}}),e(n.td,{get children(){return["Current ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"features.*"})]}}),e(n.td,{children:"-"}),e(n.td,{children:"Usages of that feature from local features and dependency feature strings"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return e(n.code,{children:"features.*.*"})}}),e(n.td,{get children(){return["Referenced feature definition in the same crate, or in the dependency crate's ",e(n.code,{children:"Cargo.toml"})]}}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return[e(n.code,{children:"dependencies.*"}),r(t),e(n.code,{children:"dev-dependencies.*"}),r(t),e(n.code,{children:"build-dependencies.*"}),r(t),e(n.code,{children:"target.*.dependencies.*"}),r(t),e(n.code,{children:"target.*.dev-dependencies.*"}),r(t),e(n.code,{children:"target.*.build-dependencies.*"})]}}),e(n.td,{get children(){return["Dependency crate's ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"package.name"}),",",r(t),"or the workspace ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"workspace.dependencies.*"})," when ",e(n.code,{children:"workspace = true"}),",",r(t),"or the current dependency entry when no more specific target exists"]}}),e(n.td,{get children(){return["Workspace ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"workspace.dependencies.*"})," when ",e(n.code,{children:"workspace = true"})]}}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return e(n.code,{children:"workspace.dependencies.*"})}}),e(n.td,{get children(){return["Target crate's ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"package.name"})," when the workspace dependency resolves to a local crate,",r(t),"or the current ",e(n.code,{children:"workspace.dependencies.*"})," entry when it does not"]}}),e(n.td,{children:"-"}),e(n.td,{get children(){return["Dependency entries in member ",e(n.code,{children:"Cargo.toml"})," files that use this workspace dependency"]}})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return[e(n.code,{children:"dependencies.*.workspace"}),r(t),e(n.code,{children:"dev-dependencies.*.workspace"}),r(t),e(n.code,{children:"build-dependencies.*.workspace"}),r(t),e(n.code,{children:"target.*.dependencies.*.workspace"}),r(t),e(n.code,{children:"target.*.dev-dependencies.*.workspace"}),r(t),e(n.code,{children:"target.*.build-dependencies.*.workspace"})]}}),e(n.td,{get children(){return["Workspace ",e(n.code,{children:"Cargo.toml"})," at the matching ",e(n.code,{children:"workspace.dependencies.*"})," entry"]}}),e(n.td,{get children(){return["Workspace ",e(n.code,{children:"Cargo.toml"})," at the matching ",e(n.code,{children:"workspace.dependencies.*"})," entry"]}}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return[e(n.code,{children:"dependencies.*.features.*"}),r(t),e(n.code,{children:"dev-dependencies.*.features.*"}),r(t),e(n.code,{children:"build-dependencies.*.features.*"}),r(t),e(n.code,{children:"target.*.dependencies.*.features.*"}),r(t),e(n.code,{children:"target.*.dev-dependencies.*.features.*"}),r(t),e(n.code,{children:"target.*.build-dependencies.*.features.*"})]}}),e(n.td,{get children(){return["Referenced dependency crate's ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"features.*"})]}}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return[e(n.code,{children:"dependencies.*.optional"}),r(t),e(n.code,{children:"dev-dependencies.*.optional"}),r(t),e(n.code,{children:"build-dependencies.*.optional"}),r(t),e(n.code,{children:"target.*.dependencies.*.optional"}),r(t),e(n.code,{children:"target.*.dev-dependencies.*.optional"}),r(t),e(n.code,{children:"target.*.build-dependencies.*.optional"})]}}),e(n.td,{get children(){return["Current dependency entry's ",e(n.code,{children:"optional"})," field"]}}),e(n.td,{children:"-"}),e(n.td,{children:"Implicit feature usages generated from this optional dependency"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"member"})}}),e(n.td,{get children(){return[e(n.code,{children:"workspace.members"}),r(t),e(n.code,{children:"workspace.members.*"})]}}),e(n.td,{get children(){return["Member crate's ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"package.name"})]}}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"member"})}}),e(n.td,{get children(){return[e(n.code,{children:"workspace.default-members"}),r(t),e(n.code,{children:"workspace.default-members.*"})]}}),e(n.td,{get children(){return["Default member crate's ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"package.name"})]}}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"path"})}}),e(n.td,{get children(){return[e(n.code,{children:"workspace.dependencies.*.path"}),r(t),e(n.code,{children:"dependencies.*.path"}),r(t),e(n.code,{children:"dev-dependencies.*.path"}),r(t),e(n.code,{children:"build-dependencies.*.path"}),r(t),e(n.code,{children:"target.*.dependencies.*.path"}),r(t),e(n.code,{children:"target.*.dev-dependencies.*.path"}),r(t),e(n.code,{children:"target.*.build-dependencies.*.path"})]}}),e(n.td,{get children(){return["Referenced local crate's ",e(n.code,{children:"Cargo.toml"})," at ",e(n.code,{children:"package.name"})]}}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"path"})}}),e(n.td,{get children(){return e(n.code,{children:"bin.*.path"})}}),e(n.td,{children:"Referenced source file"}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}})]}})]}}),`
`,e(n.p,{get children(){return["When ",e(n.code,{children:"Go to Definition"}),` returns the current location for a definition-site accessor, Tombi is intentionally signaling that there is no more specific definition target.
In editors such as VS Code, this allows `,e(n.code,{children:"editor.gotoLocation.alternativeDefinitionCommand"})," to take over and open the configured fallback, typically ",e(n.code,{children:"References"}),"."]}}),`
`,e(n.h3,{id:"code-actions",className:"group relative heading-with-anchor",children:"Code Actions"}),`
`,e(n.p,{get children(){return["When working with ",e(n.code,{children:"Cargo.toml"})," files, additional code actions are available."]}}),`
`,e(n.h4,{id:"inherit-from-workspace",className:"group relative heading-with-anchor",children:"Inherit from Workspace"}),`
`,e(n.p,{children:"Convert package fields to inherit from workspace configuration."}),`
`,e(i,{tabs:[{key:"before",label:"Before",src:"/Cargo_CodeAction_InheritFromWorkspace_Before.png",alt:"Before"},{key:"after",label:"After",src:"/Cargo_CodeAction_InheritFromWorkspace_After.png",alt:"After"}],defaultKey:"before"}),`
`,e(n.h4,{id:"inherit-dependency-from-workspace",className:"group relative heading-with-anchor",children:"Inherit Dependency from Workspace"}),`
`,e(n.p,{children:"Convert dependencies to use workspace versions."}),`
`,e(i,{tabs:[{key:"before",label:"Before",src:"/Cargo_CodeAction_InheritDependencyFromWorkspace_Before.png",alt:"Before"},{key:"after",label:"After",src:"/Cargo_CodeAction_InheritDependencyFromWorkspace_After.png",alt:"After"}],defaultKey:"before"}),`
`,e(n.h4,{id:"convert-dependency-to-table-format",className:"group relative heading-with-anchor",children:"Convert Dependency to Table Format"}),`
`,e(n.p,{children:"Transform simple string dependencies to table format for adding additional fields."}),`
`,e(i,{tabs:[{key:"before",label:"Before",src:"/Cargo_CodeAction_ConevrtDependencyToTableFormat_Before.png",alt:"Before"},{key:"after",label:"After",src:"/Cargo_CodeAction_ConevrtDependencyToTableFormat_After.png",alt:"After"}],defaultKey:"before"}),`
`,e(n.h4,{id:"add-to-workspace-and-inherit-dependency",className:"group relative heading-with-anchor",children:"Add to Workspace and Inherit Dependency"}),`
`,e(n.p,{children:"Promote versioned dependencies into the workspace while switching members to use the shared entry."}),`
`,e(i,{tabs:[{key:"before",label:"Before",src:"/Cargo_CodeAction_AddToWorkspaceAndInheritDependency_Before.png",alt:"Before"},{key:"after",label:"After",src:"/Cargo_CodeAction_AddToWorkspaceAndInheritDependency_After.png",alt:"After"}],defaultKey:"before"}),`
`,e(n.p,{children:"The workspace entry keeps the original requirement, and the member now follows the workspace-managed version."}),`
`,e(n.h4,{id:"update-dependency-to-latest-version",className:"group relative heading-with-anchor",children:"Update Dependency to Latest Version"}),`
`,e(n.p,{children:`Refresh the selected dependency requirement to the latest published version.
Use this when the manifest already references a package and you want Tombi to rewrite just the version requirement without converting the dependency shape.`}),`
`,e(n.h3,{id:"document-links",className:"group relative heading-with-anchor",children:"Document Links"}),`
`,e(n.p,{get children(){return["The extension enriches ",e(n.code,{children:"Cargo.toml"})," with contextual document links so you can jump directly to related resources:"]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["Entries in ",e(n.code,{children:"[workspace.members]"})," and ",e(n.code,{children:"[workspace.default-members]"})," open the corresponding member crate's ",e(n.code,{children:"Cargo.toml"}),"."]}}),`
`,e(n.li,{get children(){return["Dependencies in member ",e(n.code,{children:"Cargo.toml"})," files surface links to the workspace definition (when ",e(n.code,{children:"workspace = true"}),"), local path ",e(n.code,{children:"Cargo.toml"})," files, git repositories, custom registries, or crates.io when applicable."]}}),`
`,e(n.li,{get children(){return[e(n.code,{children:"[workspace.dependencies]"})," entries provide quick access both to the workspace declaration and the external source (registry/git/crates.io)."]}}),`
`,e(n.li,{get children(){return["Fields such as ",e(n.code,{children:"package.workspace"}),", ",e(n.code,{children:"lints.workspace"}),", and ",e(n.code,{children:"[bin]"})," targets link back to their workspace counterparts or the referenced source file for faster navigation."]}}),`
`]}})]}function b(c={}){const{wrapper:n}={...p(),...c.components};return n?e(n,g(c,{get children(){return e(s,c)}})):s(c)}function h(c,n){throw new Error("Expected component `"+c+"` to be defined: you likely forgot to import, pass, or provide it.")}export{b as default};
