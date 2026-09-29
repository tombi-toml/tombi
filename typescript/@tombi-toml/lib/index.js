const { existsSync } = require("node:fs");
const { join } = require("node:path");
const { platform, arch, env } = process;
// Published as both `@tombi-toml/lib` and `tombi-lib` (the release
// workflow only rewrites `name`), so read it rather than hard-coding one.
const { name: packageName } = require("./package.json");

function isMusl() {
  // `glibcVersionRuntime` is only reported by glibc-based Node.js builds.
  const report = process.report?.getReport();
  const header = typeof report === "string" ? JSON.parse(report).header : report?.header;
  return !header?.glibcVersionRuntime;
}

const PLATFORMS = {
  win32: {
    x64: "@tombi-toml/lib-win32-x64",
    arm64: "@tombi-toml/lib-win32-arm64",
  },
  darwin: {
    x64: "@tombi-toml/lib-darwin-x64",
    arm64: "@tombi-toml/lib-darwin-arm64",
  },
  linux: {
    x64: "@tombi-toml/lib-linux-x64",
    arm64: "@tombi-toml/lib-linux-arm64",
  },
  "linux-musl": {
    x64: "@tombi-toml/lib-linux-x64-musl",
    arm64: "@tombi-toml/lib-linux-arm64-musl",
  },
};

function loadBinding() {
  if (env.TOMBI_LIB_BINARY) {
    return require(env.TOMBI_LIB_BINARY);
  }

  // A local `pnpm build` output, for development within this repository.
  const localBinary = join(__dirname, "tombi-lib.node");
  if (existsSync(localBinary)) {
    return require(localBinary);
  }

  const bindingPackageName =
    platform === "linux" && isMusl()
      ? PLATFORMS["linux-musl"][arch]
      : PLATFORMS[platform]?.[arch];

  if (!bindingPackageName) {
    throw new Error(
      `${packageName} doesn't ship with a prebuilt binary for ${platform}-${arch} yet.`,
    );
  }

  try {
    return require(`${bindingPackageName}/tombi-lib.node`);
  } catch (error) {
    throw new Error(
      `The Tombi native binding "${bindingPackageName}" could not be loaded. ` +
        "The platform-specific optional package may not be installed. " +
        `Please reinstall ${packageName} with optional dependencies enabled.`,
      { cause: error },
    );
  }
}

const binding = loadBinding();

module.exports.format = binding.format;
module.exports.lint = binding.lint;
module.exports.formatSync = binding.formatSync;
module.exports.lintSync = binding.lintSync;
