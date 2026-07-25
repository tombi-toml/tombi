# Tombi for IntelliJ Platform

<!-- Plugin description -->
[Tombi][1] is a feature-rich language server for TOML.


## Usage

The plugin uses a Tombi executable from the selected project SDK, `.venv`,
`node_modules`, or `PATH`. It also searches parent directories for hoisted npm
installations. If a local executable is not available, it downloads the latest
Tombi release automatically.

You can select a specific executable in the plugin settings. Open any TOML file
to start working.


## Logging

You are strongly encouraged to enable language server logging.
This will allow corresponding logs to be recorded in log files
for further analysis should a problem arises.

Add the following line to the <b>Help</b> |
<b>Diagnostic Tools</b> | <b>Debug Log Settings</b> panel:

```text
com.intellij.platform.lsp
```


  [1]: https://tombi-toml.github.io/tombi
<!-- Plugin description end -->


## Credits

Parts of this plugin were taken or derived from:

* [@alexander-doroshko/intellij-lsp-plugin-example][3]
* [@JetBrains/intellij-community][4]
* [@JetBrains/intellij-platform-plugin-template][5]


  [3]: https://github.com/alexander-doroshko/intellij-lsp-plugin-example
  [4]: https://github.com/JetBrains/intellij-community
  [5]: https://github.com/JetBrains/intellij-platform-plugin-template
