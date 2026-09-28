import * as fs from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { format } from "node:util";

const CLI_ROOT = resolve(fileURLToPath(import.meta.url), "../..");
const PACKAGES_ROOT = resolve(CLI_ROOT, "..");
const REPO_ROOT = resolve(PACKAGES_ROOT, "../..");
const MANIFEST_PATH = resolve(CLI_ROOT, "package.json");

const rootManifest = JSON.parse(
	fs.readFileSync(MANIFEST_PATH).toString("utf-8"),
);

function getName(platform, arch, prefix = "cli") {
	return format(`${prefix}-${platform}`, arch);
}

function copyBinaryToNativePackage(
	platform,
	arch,
	{ mainPackage, prefix, binaryName, extension, main },
) {
	const os = platform.split("-")[0];
	const buildName = getName(platform, arch, prefix);
	const packageRoot = resolve(PACKAGES_ROOT, buildName);
	const packageName = `@tombi-toml/${buildName}`;

	// Update the `package.json` manifest
	const { version, license, repository, homepage } = rootManifest;
	const { engines } = readManifest(mainPackage);

	const ext = extension(os);
	const manifest = JSON.stringify(
		{
			name: packageName,
			version,
			license,
			repository,
			engines,
			homepage,
			main: main ? `${binaryName}${ext}` : undefined,
			os: [os],
			cpu: [arch],
			libc:
				os === "linux"
					? packageName.endsWith("musl")
						? ["musl"]
						: ["glibc"]
					: undefined,
		},
		null,
		2,
	);

	const manifestPath = resolve(packageRoot, "package.json");
	console.log(`Update manifest ${manifestPath}`);
	fs.writeFileSync(manifestPath, manifest);

	// Copy the binary
	const binarySource = resolve(
		REPO_ROOT,
		`${getName(platform, arch, binaryName)}${ext}`,
	);
	const binaryTarget = resolve(packageRoot, `${binaryName}${ext}`);

	if (!fs.existsSync(binarySource)) {
		console.error(
			`Source for binary for ${buildName} not found at: ${binarySource}`,
		);
		process.exit(1);
	}

	console.log(`Copy binary ${binaryTarget}`);
	fs.copyFileSync(binarySource, binaryTarget);
	fs.chmodSync(binaryTarget, 0o755);
}

function readManifest(packagePath) {
	const manifestPath = resolve(PACKAGES_ROOT, packagePath, "package.json");
	return JSON.parse(fs.readFileSync(manifestPath).toString("utf-8"));
}

function writeManifest(packagePath, targets, prefix) {
	const manifestPath = resolve(PACKAGES_ROOT, packagePath, "package.json");
	const manifestData = readManifest(packagePath);

	const nativePackages = targets.map(([platform, arch]) => [
		`@tombi-toml/${getName(platform, arch, prefix)}`,
		rootManifest.version,
	]);

	manifestData.version = rootManifest.version;
	manifestData.optionalDependencies = Object.fromEntries(nativePackages);

	console.log(`Update manifest ${manifestPath}`);
	const content = JSON.stringify(manifestData, null, 2);
	fs.writeFileSync(manifestPath, content);
}

const PLATFORMS = ["win32-%s", "darwin-%s", "linux-%s", "linux-%s-musl"];
const ARCHITECTURES = ["x64", "arm64"];

// We only publish x86_64 binaries for illumos. aarch64-unknown-illumos is a
// Tier 3 Rust target, but Rust (as of 1.97) doesn't have a prebuilt std for it
// so it cannot be built or distributed today. It is therefore excluded from the
// symmetric PLATFORMS x ARCHITECTURES matrix above.
const EXTRA_TARGETS = [["sunos-%s", "x64"]];

const TARGETS = [
	...PLATFORMS.flatMap((platform) =>
		ARCHITECTURES.map((arch) => [platform, arch]),
	),
	...EXTRA_TARGETS,
];

const PACKAGE_SETS = [
	{
		// The CLI binary (`tombi-<target>[.exe]`) dispatched by `bin/tombi`.
		mainPackage: "tombi",
		prefix: "cli",
		binaryName: "tombi",
		extension: (os) => (os === "win32" ? ".exe" : ""),
		main: false,
		targets: TARGETS,
	},
	{
		// The napi-rs addon (`tombi-lib-<target>.node`) loaded by
		// `@tombi-toml/tombi-lib`. Node.js has no illumos prebuilt addon
		// toolchain here, so it only ships the symmetric matrix.
		mainPackage: "tombi-lib",
		prefix: "lib",
		binaryName: "tombi-lib",
		extension: () => ".node",
		main: true,
		targets: TARGETS.filter(([platform]) => !platform.startsWith("sunos")),
	},
];

// The per-platform package directories are checked into git so that the set of
// published packages is visible; fail if they drift from the targets above.
function assertPackageDirectories({ prefix, targets }) {
	const expected = targets
		.map(([platform, arch]) => getName(platform, arch, prefix))
		.sort();
	const actual = fs
		.readdirSync(PACKAGES_ROOT)
		.filter((name) => name.startsWith(`${prefix}-`))
		.sort();
	if (JSON.stringify(expected) !== JSON.stringify(actual)) {
		console.error(
			`Package directories for "${prefix}-*" don't match the targets.\n` +
				`  expected: ${expected.join(", ")}\n` +
				`  actual:   ${actual.join(", ")}`,
		);
		process.exit(1);
	}
}

for (const packageSet of PACKAGE_SETS) {
	assertPackageDirectories(packageSet);
	for (const [platform, arch] of packageSet.targets) {
		copyBinaryToNativePackage(platform, arch, packageSet);
	}
	writeManifest(packageSet.mainPackage, packageSet.targets, packageSet.prefix);
}
