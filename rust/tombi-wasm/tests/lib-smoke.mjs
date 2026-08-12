import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import init, { format, lint } from "../../../typescript/@tombi-toml/wasm-lib/dist/tombi_wasm.js";

const wasm = await readFile(new URL("../../../typescript/@tombi-toml/wasm-lib/dist/tombi_wasm_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

assert.equal(await format("key={nested=1}", "playground.toml", "v1.1.0"), "key = { nested = 1 }\n");
assert.equal(await lint("key = 1", "playground.toml", "v1.1.0"), undefined);
const diagnostics = await lint("key =", "playground.toml", "v1.1.0");
assert.ok(Array.isArray(diagnostics));
assert.ok(diagnostics.length > 0);
