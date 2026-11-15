const VERSION = /^v?(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)$/;
const LATEST_PACKAGE =
  "https://registry.npmjs.org/@tombi-toml%2fwasm-lsp/latest";

export async function resolvePlaygroundRuntime(
  requested: string | null,
): Promise<string> {
  if (requested !== null) {
    const version = VERSION.exec(requested)?.[1];
    if (!version) throw new Error(`Invalid Playground version: ${requested}`);
    return version;
  }

  const response = await fetch(LATEST_PACKAGE, { cache: "no-store" });
  if (!response.ok) throw new Error("Unable to find the latest released WASM");
  const metadata: { version?: string } = await response.json();
  const version = metadata.version && VERSION.exec(metadata.version)?.[1];
  if (!version) throw new Error("Invalid latest released WASM version");
  return version;
}
