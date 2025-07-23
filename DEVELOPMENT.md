# Developer Documentation

## Debug CLI

> [!NOTE]
> The version of the built `tombi` command is `0.0.0-dev`. If you want to execute a specific version, please refer to the [Installation Document](https://tombi-toml.github.io/tombi/docs/installation).

### Use Cargo
```sh
cargo tombi
```

### Use Python
```sh
# Setup Python Environment
uv sync
source .venv/bin/activate

# Build and Run
maturin dev --uv && tombi
```

## Debug VSCode Extension
1. Select `Run and Debug` from the sidebar
2. Select `Run Extension (Debug Build)` from the dropdown
3. Press the green play button ▶️

## Working on the IntelliJ plugin

To get started, open the `editors/intellij` subdirectory in IntelliJ IDEA,
then configure Gradle.

When a pull request is opened, the plugin will be built automatically.
The plugin artifact can then be downloaded from the corresponding workflow run.

### Building the plugin

```shell
$ cd editors/intellij
$ ./gradlew buildPlugin
```

### Running tests

```shell
$ cd editors/intellij
$ ./gradlew check
```

### Resources

* [IntelliJ Platform SDK Documentation](https://plugins.jetbrains.com/docs/intellij/welcome.html)
* [JetBrains Marketplace Documentation](https://plugins.jetbrains.com/docs/marketplace/discover-jetbrains-marketplace.html)
* [InteliJ Platform Explorer](https://plugins.jetbrains.com/intellij-platform-explorer/extensions)

## toml-test

To test if it passes [toml-test](https://github.com/toml-lang/toml-test), run the following.

```sh
# Please first install toml-test
go install github.com/toml-lang/toml-test/cmd/toml-test@latest

# Run the toml-test
cargo xtask toml-test
```

## Check Performance

```sh
cargo tombi-flamegraph format
# Open the flamegraph in a browser or using a platform-specific command:
# macOS: open .tmp/flamegraph.svg
# Linux: xdg-open .tmp/flamegraph.svg
# Alternatively, manually open the file in your web browser.
```
