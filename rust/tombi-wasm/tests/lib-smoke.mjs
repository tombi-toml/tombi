import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import init, { format, lint } from "../dist/lib/tombi_wasm.js";

const wasm = await readFile(new URL("../dist/lib/tombi_wasm_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

assert.equal(await format("key={nested=1}", "playground.toml", "v1.1.0"), "key = { nested = 1 }\n");
await lint("key = 1", "playground.toml", "v1.1.0");
await assert.rejects(lint("key =", "playground.toml", "v1.1.0"));
