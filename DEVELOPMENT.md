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
source .venv/bin/activate

maturin dev --uv

tombi
```

## Debug VSCode Extension
1. Select `Run and Debug` from the sidebar
2. Select `Run Extension (Debug Build)` from the dropdown
3. Press the green play button ▶️

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
cat .tmp/flamegraph.svg
```
