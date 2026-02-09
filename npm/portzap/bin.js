#!/usr/bin/env node

const { execFileSync } = require("child_process");
const path = require("path");
const os = require("os");

const PLATFORMS = {
  "darwin-arm64": "@portzap/darwin-arm64",
  "darwin-x64": "@portzap/darwin-x64",
  "linux-x64": "@portzap/linux-x64",
  "linux-arm64": "@portzap/linux-arm64",
};

function getBinaryPath() {
  const platform = os.platform();
  const arch = os.arch();
  const key = `${platform}-${arch}`;
  const pkg = PLATFORMS[key];

  if (!pkg) {
    console.error(
      `portzap: unsupported platform ${platform}-${arch}\n` +
        `Supported: ${Object.keys(PLATFORMS).join(", ")}`
    );
    process.exit(1);
  }

  try {
    const pkgDir = path.dirname(require.resolve(`${pkg}/package.json`));
    const binName = platform === "win32" ? "portzap.exe" : "portzap";
    return path.join(pkgDir, binName);
  } catch {
    console.error(
      `portzap: failed to find binary package ${pkg}\n` +
        `Try reinstalling: npm install portzap`
    );
    process.exit(1);
  }
}

const binPath = getBinaryPath();
const args = process.argv.slice(2);

try {
  execFileSync(binPath, args, { stdio: "inherit" });
} catch (err) {
  if (err.status !== null) {
    process.exit(err.status);
  }
  throw err;
}
