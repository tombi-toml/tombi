const version = new URL(self.location.href).searchParams.get("version");
if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version || "")) {
  throw new Error("Invalid Playground WASM version");
}

await import(
  `https://cdn.jsdelivr.net/npm/@tombi-toml/wasm-lsp@${version}/dist/worker.js`
);
