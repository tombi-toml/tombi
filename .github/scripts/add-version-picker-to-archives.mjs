import { readFile, readdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const pages = process.argv[2];
if (!pages) throw new Error("Usage: node add-version-picker-to-archives.mjs PAGES_DIR");

const archiveScript = '<script type="module" src="/tombi/common/v1/archive/archive.js"></script>';
const picker =
  '<link rel="stylesheet" href="/tombi/common/v1/version-picker.css">' +
  archiveScript +
  '<script defer src="/tombi/common/v1/version-picker.js"></script>';

async function update(directory) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      await update(path);
    } else if (entry.name.endsWith(".html")) {
      const html = await readFile(path, "utf8");
      if (html.includes(archiveScript) && !html.includes("version-picker.js")) {
        await writeFile(path, html.replace(archiveScript, picker));
      }
    }
  }
}

for (const entry of await readdir(pages, { withFileTypes: true })) {
  if (entry.isDirectory() && /^v\d+\.\d+\.\d+(?:[.-][\dA-Za-z.-]+)?$/.test(entry.name)) {
    await update(join(pages, entry.name));
  }
}
