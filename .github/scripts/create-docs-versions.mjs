import { readdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const pages = process.argv[2];
if (!pages) throw new Error("Usage: node create-docs-versions.mjs PAGES_DIR");

const versionPattern = /^v(\d+)\.(\d+)\.(\d+)(?:[.-]([\dA-Za-z.-]+))?$/;
function compareVersions(left, right) {
  const a = versionPattern.exec(left);
  const b = versionPattern.exec(right);
  for (let index = 1; index <= 3; index++) {
    const difference = BigInt(a[index]) - BigInt(b[index]);
    if (difference) return difference > 0 ? 1 : -1;
  }
  if (!a[4] || !b[4]) return Number(Boolean(b[4])) - Number(Boolean(a[4]));
  const aParts = a[4].split(".");
  const bParts = b[4].split(".");
  for (let index = 0; index < Math.min(aParts.length, bParts.length); index++) {
    const aPart = aParts[index];
    const bPart = bParts[index];
    if (aPart === bPart) continue;
    const aNumber = /^\d+$/.test(aPart);
    const bNumber = /^\d+$/.test(bPart);
    if (aNumber && bNumber) return BigInt(aPart) > BigInt(bPart) ? 1 : -1;
    if (aNumber !== bNumber) return aNumber ? -1 : 1;
    return aPart > bPart ? 1 : -1;
  }
  return Math.sign(aParts.length - bParts.length);
}

const entries = await readdir(pages, { withFileTypes: true });
const versions = entries
  .filter(
    (entry) =>
      entry.isDirectory() && versionPattern.test(entry.name),
  )
  .map((entry) => entry.name)
  .sort((a, b) => compareVersions(b, a));

await writeFile(join(pages, "versions.json"), JSON.stringify(["main", ...versions]));
