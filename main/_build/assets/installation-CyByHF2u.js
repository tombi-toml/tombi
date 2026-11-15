import{M as C}from"./index-6OQr06v0.js";import{h as A,e as L,A as O,i as y,q as g,l as u,b as e,F as j,m as V,v as S,w as B,x as T,r as D,t as f,k as G}from"./web-DCz9_uPG.js";import{C as z}from"./CodeBlock-DcLPlfzH.js";import{u as F}from"./CodeTabsContext-Bq8doJbE.js";import"./index-Dhcltlfs.js";import"./index-7xQSF2zI.js";var J=f("<div><!$><!/><!$><!/>"),X=f("<button type=button style=min-width:64px;height:40px><!$><!/><!$><!/>"),Z=f('<div class="absolute bottom-0 left-0 w-full h-1 bg-tombi-700 dark:bg-yellow">');function l(a){const t=F(),[o,h]=L(a.defaultKey||a.tabs[0].key),n=()=>a.tabs.find(i=>i.key===o());O(()=>{const i=t?.selectedKey();i&&a.tabs.some(c=>c.key===i)&&h(i)});const m=i=>{h(i),t?.selectKey(i)};return(()=>{var i=y(J),c=i.firstChild,[p,v]=g(c.nextSibling),$=p.nextSibling,[M,P]=g($.nextSibling);return u(i,e(j,{get each(){return a.tabs},children:d=>(()=>{var r=y(X),_=r.firstChild,[k,E]=g(_.nextSibling),I=k.nextSibling,[H,K]=g(I.nextSibling);return r.$$click=()=>m(d.key),u(r,()=>d.label,k,E),u(r,(()=>{var s=V(()=>o()===d.key);return()=>s()&&y(Z)})(),H,K),S(s=>{var x=`px-4 font-semibold text-base cursor-pointer bg-transparent border-0 relative transition-colors
                ${o()===d.key?"text-gray-800 dark:text-gray-100":"text-gray-500 dark:text-gray-400"}
                focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-600 dark:focus-visible:ring-blue-400 focus-visible:ring-offset-2 focus-visible:rounded
              `,w=d.key;return x!==s.e&&B(r,s.e=x),w!==s.t&&T(r,"data-key",s.t=w),s},{e:void 0,t:void 0}),D(),r})()}),p,v),u(i,e(z,{get code(){return n()?.command||""},get language(){return n()?.language||a.language}}),M,P),S(()=>T(i,"data-archive-code-tabs",JSON.stringify(a.tabs))),i})()}A(["click"]);const[te,q]=L(null);function N(a){const t={a:"a",br:"br",code:"code",h1:"h1",h2:"h2",h3:"h3",p:"p",...C(),...a.components},{CodeBlock:o,InstallationMethodGrid:h,LinkCard:n,PageHeading:m}=t;return o||b("CodeBlock"),h||b("InstallationMethodGrid"),n||b("LinkCard"),m||b("PageHeading"),[e(m,{title:"Installation",description:"Install via CLI, winget, Homebrew, pip, npm, or IDE extensions.",og_url:"https://tombi-toml.github.io/tombi/main/docs/installation"}),`
`,`
`,`
`,e(t.h1,{id:"installation",className:"group relative heading-with-anchor",children:"Installation"}),`
`,e(t.p,{children:"Choose your preferred installation method:"}),`
`,e(h,{onSelect:i=>{console.log("method",i),q(i);const c=document.getElementById(i);if(c){const v=c.offsetTop-80;window.scrollTo({top:v,behavior:"smooth"})}}}),`
`,e(t.h2,{id:"cli",className:"group relative heading-with-anchor",children:"CLI"}),`
`,e(l,{tabs:[{key:"sh",label:"Shell Script",command:"curl -fsSL https://tombi-toml.github.io/tombi/install.sh | sh",language:"sh"},{key:"powershell",label:"PowerShell",command:"& ([scriptblock]::Create((irm https://tombi-toml.github.io/tombi/install.ps1)))",language:"powershell"}],defaultKey:"sh",language:"sh"}),`
`,e(t.p,{get children(){return["On Windows, the default installation directory is ",e(t.code,{children:"%LOCALAPPDATA%\\tombi\\bin"}),`, and
the PowerShell installer does not modify your `,e(t.code,{children:"PATH"}),`; add the directory to your
user `,e(t.code,{children:"PATH"}),", or use the returned executable path directly."]}}),`
`,e(t.p,{children:"Install a specific version:"}),`
`,e(l,{tabs:[{key:"sh",label:"Shell Script",command:"curl -fsSL https://tombi-toml.github.io/tombi/install.sh | sh -s -- --version 0.7.5",language:"sh"},{key:"powershell",label:"PowerShell",command:"& ([scriptblock]::Create((irm https://tombi-toml.github.io/tombi/install.ps1))) -Version 0.7.5",language:"powershell"}],defaultKey:"sh",language:"sh"}),`
`,e(t.p,{get children(){return["Install to a specific directory:",e(t.br,{}),`
`,"(For the Shell Script installer, specify a directory that is already included in your ",e(t.code,{children:"PATH"}),".)"]}}),`
`,e(l,{tabs:[{key:"sh",label:"Shell Script",command:"curl -fsSL https://tombi-toml.github.io/tombi/install.sh | sh -s -- --install-dir /custom/path",language:"sh"},{key:"powershell",label:"PowerShell",command:'& ([scriptblock]::Create((irm https://tombi-toml.github.io/tombi/install.ps1))) -InstallDir "C:\\Tools\\tombi"',language:"powershell"}],defaultKey:"sh",language:"sh"}),`
`,e(t.p,{children:"Verify the downloaded archive checksum before extraction:"}),`
`,e(l,{tabs:[{key:"sh",label:"Shell Script",command:"curl -fsSL https://tombi-toml.github.io/tombi/install.sh | sh -s -- --checksum <sha256>",language:"sh"},{key:"powershell",label:"PowerShell",command:"& ([scriptblock]::Create((irm https://tombi-toml.github.io/tombi/install.ps1))) -Checksum <sha256>",language:"powershell"}],defaultKey:"sh",language:"sh"}),`
`,e(t.h2,{id:"windows-package-manager",className:"group relative heading-with-anchor",children:"Windows Package Manager"}),`
`,e(o,{code:"winget install --id tombi-toml.tombi --exact",language:"powershell"}),`
`,e(t.h2,{id:"homebrew",className:"group relative heading-with-anchor",children:"Homebrew"}),`
`,e(o,{code:"brew install tombi",language:"sh"}),`
`,e(t.h2,{id:"python",className:"group relative heading-with-anchor",children:"Python"}),`
`,e(t.h3,{id:"quick-try",className:"group relative heading-with-anchor",children:"Quick Try"}),`
`,e(l,{tabs:[{key:"uvx",label:"uvx",command:"uvx tombi"}],defaultKey:"uvx",language:"bash"}),`
`,e(t.h3,{id:"project-installation",className:"group relative heading-with-anchor",children:"Project Installation"}),`
`,e(l,{tabs:[{key:"uv",label:"uv",command:"uv add --dev tombi"},{key:"poetry",label:"poetry",command:"poetry add --dev tombi"},{key:"hatch",label:"hatch",command:"hatch add --dev tombi"},{key:"pip",label:"pip",command:"pip install tombi"}],defaultKey:"uv",language:"bash"}),`
`,e(t.h3,{id:"global-installation",className:"group relative heading-with-anchor",children:"Global Installation"}),`
`,e(l,{tabs:[{key:"uv",label:"uv",command:"uv tool install tombi"},{key:"pipx",label:"pipx",command:"pipx install tombi"}],defaultKey:"uv",language:"bash"}),`
`,e(t.h2,{id:"java-script",className:"group relative heading-with-anchor",children:"JavaScript"}),`
`,e(t.h3,{id:"quick-try",className:"group relative heading-with-anchor",children:"Quick Try"}),`
`,e(l,{tabs:[{key:"npx",label:"npx",command:"npx tombi"}],defaultKey:"npx",language:"bash"}),`
`,e(t.h3,{id:"project-installation",className:"group relative heading-with-anchor",children:"Project Installation"}),`
`,e(l,{tabs:[{key:"npm",label:"npm",command:"npm install --save-dev tombi"},{key:"pnpm",label:"pnpm",command:"pnpm add --save-dev tombi"},{key:"yarn",label:"yarn",command:"yarn add --dev tombi"}],defaultKey:"npm",language:"bash"}),`
`,e(t.h3,{id:"global-installation",className:"group relative heading-with-anchor",children:"Global Installation"}),`
`,e(l,{tabs:[{key:"npm",label:"npm",command:"npm install -g tombi"},{key:"pnpm",label:"pnpm",command:"pnpm add -g tombi"},{key:"yarn",label:"yarn",command:"yarn global add tombi"}],defaultKey:"npm",language:"bash"}),`
`,e(t.h2,{id:"vscode",className:"group relative heading-with-anchor",children:"VSCode"}),`
`,e(n,{title:"Tombi - Visual Studio Marketplace",description:"Extension for Visual Studio Code - TOML Language Server",image:"/vscode.svg",url:"https://marketplace.visualstudio.com/items?itemName=tombi-toml.tombi"}),`
`,e(t.h2,{id:"open-vsx",className:"group relative heading-with-anchor",children:"Open VSX"}),`
`,e(t.p,{children:"Cursor, Windsurf, etc."}),`
`,e(n,{title:"Tombi - Open VSX Registry",description:"Extension for Open VSX - TOML Language Server",image:"/openvsx.png",url:"https://open-vsx.org/extension/tombi-toml/tombi"}),`
`,e(t.h2,{id:"zed",className:"group relative heading-with-anchor",children:"Zed"}),`
`,e(n,{title:"Tombi - Zed Extensions",description:"Extension for Zed - TOML Language Server",image:"/zed.jpeg",url:"https://zed.dev/extensions?query=tombi&filter=language-servers"}),`
`,e(t.p,{get children(){return[`The Tombi extension currently published in Zed's extension registry focuses on Language Server Protocol (LSP) features.
For syntax highlighting and other syntax-related editor features, also install the `,e(t.a,{href:"https://github.com/zed-extensions/toml",children:"TOML extension"}),`.
For Tombi-specific configuration, see the `,e(t.a,{href:"/tombi/main/docs/editors/zed-extension",children:"Zed Extension"})," page."]}}),`
`,e(t.h2,{id:"helix",className:"group relative heading-with-anchor",children:"Helix"}),`
`,e(t.p,{get children(){return["Install the tombi binary locally and append the following ",e(t.a,{href:"https://docs.helix-editor.com/configuration.html",children:"configuration"}),":"]}}),`
`,e(o,{code:`[[language]]
name = "toml"
auto-format = true
language-servers = ["tombi"]`,language:"toml"}),`
`,e(t.h2,{id:"jet-brains-ide",className:"group relative heading-with-anchor",children:"JetBrains IDE"}),`
`,e(n,{title:"Tombi - JetBrains Marketplace",description:"Plugin for JetBrains IDE - TOML Language Server",image:"/jetbrains.png",url:"https://plugins.jetbrains.com/plugin/28017-tombi"}),`
`,e(t.h2,{id:"neovim",className:"group relative heading-with-anchor",children:"Neovim"}),`
`,e(n,{title:"Tombi - Neovim",description:"Neovim - TOML Language Server",image:"/neovim.png",url:"https://github.com/neovim/nvim-lspconfig/blob/master/lsp/tombi.lua"}),`
`,e(t.h2,{id:"emacs",className:"group relative heading-with-anchor",children:"Emacs"}),`
`,e(n,{title:"Tombi - Emacs",description:"Emacs - TOML Language Server",image:"/emacs.png",url:"https://emacs-lsp.github.io/lsp-mode/page/lsp-toml-tombi/"}),`
`,e(t.h2,{id:"git-hub-actions",className:"group relative heading-with-anchor",children:"GitHub Actions"}),`
`,e(n,{title:"Tombi - GitHub Action",description:"GitHub Action to install Tombi",image:"/github-action.png",url:"https://github.com/tombi-toml/setup-tombi"})]}function ie(a={}){const{wrapper:t}={...C(),...a.components};return t?e(t,G(a,{get children(){return e(N,a)}})):N(a)}function b(a,t){throw new Error("Expected component `"+a+"` to be defined: you likely forgot to import, pass, or provide it.")}export{ie as default,te as selectedMethod,q as setSelectedMethod};
