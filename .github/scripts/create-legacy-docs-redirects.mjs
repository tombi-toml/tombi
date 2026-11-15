import { mkdir, readdir, writeFile } from "node:fs/promises";
import { dirname, join, relative } from "node:path";

const site = process.argv[2];
if (!site) throw new Error("Usage: node create-legacy-docs-redirects.mjs SITE");

const main = join(site, "main");
async function indexFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  return (
    await Promise.all(
      entries.map((entry) => {
        const path = join(directory, entry.name);
        return entry.isDirectory()
          ? indexFiles(path)
          : entry.name === "index.html"
            ? [path]
            : [];
      }),
    )
  ).flat();
}

for (const source of await indexFiles(main)) {
  const route = relative(main, source);
  if (route === "index.html") continue;

  const target = `/tombi/main/${route.slice(0, -"index.html".length)}`;
  const destination = join(site, route);
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(
    destination,
    `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta http-equiv="refresh" content="0; url=${target}"><title>Redirecting to Tombi</title><script>location.replace(${JSON.stringify(target)} + location.search + location.hash)</script></head><body><a href="${target}">Continue to Tombi</a></body></html>`,
  );
}
