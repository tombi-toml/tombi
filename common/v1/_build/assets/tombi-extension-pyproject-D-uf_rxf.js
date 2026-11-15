import{b as e,k as h,i as r,t as p}from"./web-DCz9_uPG.js";import{M as s}from"./index-6OQr06v0.js";import{W as u,N as g}from"./Highlight-CMhf5ZLw.js";import{I as d}from"./ImageTabs-ByVWvQJA.js";var t=p("<br>");function a(c){const n={a:"a",code:"code",h1:"h1",h2:"h2",h3:"h3",h4:"h4",li:"li",p:"p",table:"table",tbody:"tbody",td:"td",th:"th",thead:"thead",tr:"tr",ul:"ul",...s(),...c.components},{CodeBlock:o,PageHeading:i}=n;return o||l("CodeBlock"),i||l("PageHeading"),[e(i,{title:"Pyproject Extension",description:"Python pyproject.toml specific features and validations.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/extensions/tombi-extension-pyproject"}),`
`,`
`,e(n.h1,{id:"pyproject-extension",className:"group relative heading-with-anchor",children:"Pyproject Extension"}),`
`,e(n.p,{get children(){return["Tombi provides enhanced features for Python projects through ",e(n.code,{children:"pyproject.toml"}),", with special support for uv workspace and source metadata."]}}),`
`,e(n.p,{get children(){return["Experimentally, we provide additional features when editing ",e(n.code,{children:"pyproject.toml"})," files."]}}),`
`,e(u,{get children(){return e(n.p,{get children(){return["These features are designed around the Python packaging ecosystem and currently understand uv-specific workspace conventions such as ",e(n.code,{children:"[tool.uv.workspace]"})," and ",e(n.code,{children:"[tool.uv.sources]"}),"."]}})}}),`
`,e(n.h2,{id:"supported-features",className:"group relative heading-with-anchor",children:"Supported Features"}),`
`,e(n.h3,{id:"packaging-metadata-diagnostics",className:"group relative heading-with-anchor",children:"Packaging Metadata Diagnostics"}),`
`,e(n.p,{get children(){return["When linting a ",e(n.code,{children:"pyproject.toml"}),", Tombi checks static metadata rules from the Python packaging specifications in addition to JSON Schema validation. It parses package requirements in ",e(n.code,{children:"build-system.requires"}),", ",e(n.code,{children:"project.dependencies"}),", ",e(n.code,{children:"project.optional-dependencies"}),", and ",e(n.code,{children:"dependency-groups"})," using PEP 508. It applies the same grammar to uv's constraint, override, and build-constraint dependency arrays. It checks classifier names against a bundled snapshot of ",e(n.a,{href:"https://github.com/pypa/trove-classifiers",children:"PyPA's canonical Trove classifier list"})," and permits private classifiers beginning with ",e(n.code,{children:"Private ::"}),". It also reports invalid email display names, invalid entry point identifiers, invalid import names, duplicate declarations, names declared as both exclusive and shared, and missing intermediate parent names. Dependency-group and extra names use PEP 503 normalization for comparisons. Tombi reports undefined group includes, normalized duplicate names, and include cycles. Unknown classifiers, duplicate import names, and missing import parents are warnings. Invalid names, requirements, ambiguous imports, undefined or cyclic group includes, and normalized duplicate group or extra names are errors."]}}),`
`,e(n.p,{get children(){return["These checks use only the file contents and run offline. They do not inspect project code or build artifacts. The classifier list is pinned to the revision recorded in ",e(n.code,{children:"extensions/tombi-extension-pyproject/data/trove-classifiers.txt"}),"."]}}),`
`,e(n.p,{get children(){return["See the ",e(n.a,{href:"https://packaging.python.org/en/latest/specifications/pyproject-toml/#import-names",children:"pyproject.toml specification"})," and ",e(n.a,{href:"https://peps.python.org/pep-0794/",children:"PEP 794"}),"."]}}),`
`,e(n.h3,{id:"completion",className:"group relative heading-with-anchor",children:"Completion"}),`
`,e(n.p,{get children(){return["The extension provides path-aware completion in places where ",e(n.code,{children:"pyproject.toml"})," usually references local files or member projects."]}}),`
`,e(o,{code:`[tool.uv.workspace]
members = ["packages/api"] # <- directory suggestions

[tool.uv.sources]
my-lib = { path = "packages/my-lib" } # <- file/directory suggestions

[project]
readme = "README.md" # <- file suggestions
license = { file = "LICENSE" } # <- file suggestions
license-files = ["licenses/*.txt"] # <- file suggestions

[build-system]
backend-path = ["backend"] # <- directory/file suggestions`,language:"toml"}),`
`,e(n.p,{children:"Supported path completion targets include:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return e(n.code,{children:"[tool.uv.workspace].members"})}}),`
`,e(n.li,{get children(){return e(n.code,{children:"[tool.uv.workspace].exclude"})}}),`
`,e(n.li,{get children(){return e(n.code,{children:"[tool.uv.sources].*.path"})}}),`
`,e(n.li,{get children(){return e(n.code,{children:"build-system.backend-path"})}}),`
`,e(n.li,{get children(){return e(n.code,{children:"project.readme"})}}),`
`,e(n.li,{get children(){return e(n.code,{children:"project.license.file"})}}),`
`,e(n.li,{get children(){return e(n.code,{children:"project.license-files"})}}),`
`]}}),`
`,e(n.h3,{id:"hover",className:"group relative heading-with-anchor",children:"Hover"}),`
`,e(n.p,{children:"For dependency entries, Tombi enriches hover content with package metadata."}),`
`,e(o,{code:`[project]
dependencies = [
  "requests>=2.0", # <- package name and summary from PyPI
]

[tool.uv.sources]
my-lib = { workspace = true } # <- package name and description from the local member`,language:"toml"}),`
`,e(n.p,{children:"Behavior:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["workspace and local path dependencies show metadata from the referenced member ",e(n.code,{children:"pyproject.toml"})]}}),`
`,e(n.li,{children:"registry dependencies can show package metadata fetched from PyPI"}),`
`,e(n.li,{children:"in offline mode, remote PyPI lookups are skipped and only local workspace/path metadata is used"}),`
`]}}),`
`,e(n.h3,{id:"inlay-hints",className:"group relative heading-with-anchor",children:"Inlay Hints"}),`
`,e(n.p,{get children(){return["The extension can show inline hints for workspace-managed dependency versions in ",e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(n.p,{children:"These hints make it easier to confirm which shared dependency version is being applied from the workspace without manually tracing the workspace configuration."}),`
`,e(n.h3,{id:"go-to-definition",className:"group relative heading-with-anchor",children:"Go to Definition"}),`
`,e(n.p,{get children(){return["For example, suppose you have a ",e(n.code,{children:"pyproject.toml"})," like the one below:"]}}),`
`,e(n.h4,{id:"go-to-package-definition-from-dependencies",className:"group relative heading-with-anchor",get children(){return["Go to package definition from ",e(n.code,{children:"dependencies"})]}}),`
`,e(o,{code:`[project]
name = "albatross"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = ["bird-feeder", "tqdm>=4,<5"]

[tool.uv.sources]
bird-feeder = { workspace = true }`,language:"toml"}),`
`,e(n.p,{get children(){return["In this case, when your cursor is on ",e(n.code,{children:"workspace"}),`, executing "Go to Definition" will navigate you to the corresponding workspace-member entry in the workspace's `,e(n.code,{children:"pyproject.toml"}),"."]}}),`
`,e(n.p,{get children(){return["When your cursor is on the source key itself, such as ",e(n.code,{children:"bird-feeder"}),`, "Go to Definition" navigates to the member package's `,e(n.code,{children:"pyproject.toml"})," and lands on its ",e(n.code,{children:"project.name"})," value."]}}),`
`,e(g,{get children(){return e(n.p,{get children(){return["The same entry in the workspace ",e(n.code,{children:"pyproject.toml"}),' is used for "Go to Declaration".']}})}}),`
`,e(n.h3,{id:"go-to-declaration",className:"group relative heading-with-anchor",children:"Go to Declaration"}),`
`,e(o,{code:`[project]
name = "albatross"
version = "0.1.0"
dependencies = ["bird-feeder", "tqdm>=4,<5"]

[tool.uv.sources]
bird-feeder = { workspace = true }`,language:"toml"}),`
`,e(n.p,{get children(){return["In this case, when your cursor is on ",e(n.code,{children:"workspace"}),`, executing "Go to Declaration" will navigate you to the workspace definition in the workspace's `,e(n.code,{children:"pyproject.toml"})," file."]}}),`
`,e(n.p,{get children(){return[e(n.code,{children:"Go to Declaration"})," also works when the cursor is placed on the dependency string itself (for example ",e(n.code,{children:'"bird-feeder"'}),`).
If the package comes from `,e(n.code,{children:"tool.uv.sources"})," with ",e(n.code,{children:"workspace = true"}),", the command jumps to the member entry in the workspace ",e(n.code,{children:"pyproject.toml"}),", letting you inspect which pattern pulled the project into the workspace."]}}),`
`,e(n.h3,{id:"navigation-list",className:"group relative heading-with-anchor",children:"Navigation List"}),`
`,e(n.p,{get children(){return["The table below describes the intended accessor-by-accessor split between ",e(n.code,{children:"Go to Definition"}),", ",e(n.code,{children:"Go to Declaration"}),", and ",e(n.code,{children:"Find References"}),"."]}}),`
`,e(n.p,{get children(){return["For dependency strings, ",e(n.code,{children:"Go to Definition"})," follows a compact rule:"]}}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{children:"stay on the current string when this file itself is the source of truth for the PyPI dependency"}),`
`,e(n.li,{children:"jump to the workspace dependency entry when the version is inherited from the workspace"}),`
`,e(n.li,{get children(){return["jump to the target package's ",e(n.code,{children:"project.name"})," when the dependency resolves to a workspace member or local source"]}}),`
`]}}),`
`,e(n.table,{get children(){return[e(n.thead,{get children(){return e(n.tr,{get children(){return[e(n.th,{children:"type"}),e(n.th,{children:"Accessors"}),e(n.th,{children:"Go to Definition"}),e(n.th,{children:"Go to Declaration"}),e(n.th,{children:"Find References"})]}})}}),e(n.tbody,{get children(){return[e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return e(n.code,{children:"project.name"})}}),e(n.td,{get children(){return["Current ",e(n.code,{children:"pyproject.toml"})," at ",e(n.code,{children:"project.name"})]}}),e(n.td,{children:"-"}),e(n.td,{get children(){return["Usages of this package from dependency strings and ",e(n.code,{children:"tool.uv.sources.*"})," entries"]}})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return[e(n.code,{children:"project.dependencies.*"}),r(t),e(n.code,{children:"project.optional-dependencies.*.*"}),r(t),e(n.code,{children:"dependency-groups.*.*"})]}}),e(n.td,{get children(){return["Current dependency string,",r(t),"or the workspace ",e(n.code,{children:"pyproject.toml"})," dependency entry that provides its version,",r(t)," or the target package's ",e(n.code,{children:"pyproject.toml"})," at ",e(n.code,{children:"project.name"})," when it resolves to a workspace member",r(t),"or local source"]}}),e(n.td,{get children(){return["The entry that defines the dependency, such as ",e(n.code,{children:"tool.uv.sources.*"})," in the current ",e(n.code,{children:"pyproject.toml"}),r(t),"or a workspace dependency entry in the workspace ",e(n.code,{children:"pyproject.toml"})]}}),e(n.td,{children:"Other dependency-string usages of the same package"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return e(n.code,{children:"dependency-groups.*"})}}),e(n.td,{get children(){return["Current ",e(n.code,{children:"dependency-groups.*"})," key"]}}),e(n.td,{children:"-"}),e(n.td,{get children(){return[e(n.code,{children:"include-group"})," entries that reference this dependency group"]}})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"dependency"})}}),e(n.td,{get children(){return e(n.code,{children:"dependency-groups.*.*.include-group"})}}),e(n.td,{get children(){return["Referenced ",e(n.code,{children:"dependency-groups.*"})," key in the same ",e(n.code,{children:"pyproject.toml"})]}}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"member"})}}),e(n.td,{get children(){return[e(n.code,{children:"tool.uv.workspace.members"}),r(t),e(n.code,{children:"tool.uv.workspace.members.*"})]}}),e(n.td,{get children(){return["Member package's ",e(n.code,{children:"pyproject.toml"})," at ",e(n.code,{children:"project.name"})]}}),e(n.td,{get children(){return["Member package's ",e(n.code,{children:"pyproject.toml"})," at ",e(n.code,{children:"project.name"})]}}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"member"})}}),e(n.td,{get children(){return e(n.code,{children:"tool.uv.sources.*"})}}),e(n.td,{get children(){return["Member package's ",e(n.code,{children:"pyproject.toml"})," at ",e(n.code,{children:"project.name"}),",",r(t),"or the referenced local project's ",e(n.code,{children:"pyproject.toml"})," at ",e(n.code,{children:"project.name"})]}}),e(n.td,{get children(){return["Workspace ",e(n.code,{children:"pyproject.toml"})," at the matching ",e(n.code,{children:"tool.uv.workspace.members.*"})," entry when the source is workspace-managed"]}}),e(n.td,{children:"Dependency strings and source entries that use this package"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"member"})}}),e(n.td,{get children(){return e(n.code,{children:"tool.uv.sources.*.workspace"})}}),e(n.td,{get children(){return["Workspace ",e(n.code,{children:"pyproject.toml"})," at the matching ",e(n.code,{children:"tool.uv.workspace.members.*"})," entry"]}}),e(n.td,{get children(){return["Workspace ",e(n.code,{children:"pyproject.toml"})," at the matching ",e(n.code,{children:"tool.uv.workspace.members.*"})," entry"]}}),e(n.td,{children:"Dependency strings and source entries that use this package"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"path"})}}),e(n.td,{get children(){return e(n.code,{children:"tool.uv.sources.*.path"})}}),e(n.td,{get children(){return["Referenced local project's ",e(n.code,{children:"pyproject.toml"})," at ",e(n.code,{children:"project.name"})]}}),e(n.td,{children:"-"}),e(n.td,{children:"Dependency strings and source entries that use this package"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"path"})}}),e(n.td,{get children(){return[e(n.code,{children:"project.readme"}),r(t),e(n.code,{children:"project.readme.file"})]}}),e(n.td,{children:"Referenced file"}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"path"})}}),e(n.td,{get children(){return e(n.code,{children:"project.license.file"})}}),e(n.td,{children:"Referenced file"}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"path"})}}),e(n.td,{get children(){return e(n.code,{children:"project.license-files.*"})}}),e(n.td,{children:"Referenced file, or each file matched by the glob"}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}}),e(n.tr,{get children(){return[e(n.td,{get children(){return e(n.code,{children:"path"})}}),e(n.td,{get children(){return e(n.code,{children:"build-system.backend-path.*"})}}),e(n.td,{children:"Referenced backend file or directory"}),e(n.td,{children:"-"}),e(n.td,{children:"-"})]}})]}})]}}),`
`,e(n.p,{get children(){return["When ",e(n.code,{children:"Go to Definition"}),` returns the current location for a definition-site accessor, Tombi is intentionally signaling that there is no more specific definition target.
In editors such as VS Code, this allows `,e(n.code,{children:"editor.gotoLocation.alternativeDefinitionCommand"})," to take over and open the configured fallback, typically ",e(n.code,{children:"References"}),"."]}}),`
`,e(n.h3,{id:"code-actions",className:"group relative heading-with-anchor",children:"Code Actions"}),`
`,e(n.p,{get children(){return["When editing member ",e(n.code,{children:"pyproject.toml"})," files, additional refactor actions help you converge on workspace-managed dependencies."]}}),`
`,e(n.h4,{id:"use-workspace-dependency",className:"group relative heading-with-anchor",children:"Use Workspace Dependency"}),`
`,e(n.p,{children:'If the workspace already pins the dependency, select "Use Workspace Dependency" to strip the version requirement (and keep extras) so the member follows the workspace definition.'}),`
`,e(d,{tabs:[{key:"before",label:"Before",src:"/uv_CodeAction_UseWorkspaceDependency_Before.png",alt:"Before"},{key:"after",label:"After",src:"/uv_CodeAction_UseWorkspaceDependency_After.png",alt:"After"}],defaultKey:"before"}),`
`,e(n.p,{get children(){return["The action also works inside ",e(n.code,{children:"[project.optional-dependencies.*]"})," groups."]}}),`
`,e(n.h4,{id:"add-to-workspace-and-use-workspace-dependency",className:"group relative heading-with-anchor",children:"Add to Workspace and Use Workspace Dependency"}),`
`,e(n.p,{get children(){return["When the workspace has not registered the dependency yet, this action adds the current versioned requirement to the workspace's ",e(n.code,{children:"[project.dependencies]"})," array and rewrites the member entry to use the shared dependency."]}}),`
`,e(d,{tabs:[{key:"before",label:"Before",src:"/uv_CodeAction_AddToWorkspaceAndUseWorkspaceDependency_Before.png",alt:"Before"},{key:"after",label:"After",src:"/uv_CodeAction_AddToWorkspaceAndUseWorkspaceDependency_After.png",alt:"After"}],defaultKey:"before"}),`
`,e(n.h4,{id:"update-dependency-to-latest-version",className:"group relative heading-with-anchor",children:"Update Dependency to Latest Version"}),`
`,e(n.p,{children:`Refresh the selected dependency requirement to the latest published version.
Use this when a dependency string is already present and you want Tombi to rewrite the pinned requirement without changing the surrounding dependency group structure.`}),`
`,e(n.h3,{id:"document-links",className:"group relative heading-with-anchor",children:"Document Links"}),`
`,e(n.p,{children:"The extension annotates key project metadata with document links so you can jump to the right file or website in one click:"}),`
`,e(n.ul,{get children(){return[`
`,e(n.li,{get children(){return["entries in ",e(n.code,{children:"[tool.uv.workspace.members]"})," and matches found via globs open the associated member project's ",e(n.code,{children:"pyproject.toml"})]}}),`
`,e(n.li,{get children(){return["sources declared under ",e(n.code,{children:"[tool.uv.sources]"})," with ",e(n.code,{children:"workspace = true"})," expose links to both the member ",e(n.code,{children:"pyproject.toml"})," and the workspace root ",e(n.code,{children:"pyproject.toml"})]}}),`
`,e(n.li,{get children(){return["dependency arrays in ",e(n.code,{children:"[project.dependencies]"}),", ",e(n.code,{children:"[project.optional-dependencies.*]"}),", and ",e(n.code,{children:"[dependency-groups.*]"})," link to local workspace packages when available, or fall back to the package page on PyPI"]}}),`
`]}})]}function w(c={}){const{wrapper:n}={...s(),...c.components};return n?e(n,h(c,{get children(){return e(a,c)}})):a(c)}function l(c,n){throw new Error("Expected component `"+c+"` to be defined: you likely forgot to import, pass, or provide it.")}export{w as default};
