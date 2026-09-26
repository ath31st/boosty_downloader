import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SEMVER = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[\w.-]+)?$/;

const CARGO_FILES = ["core/Cargo.toml", "src-tauri/Cargo.toml"];
const TAURI_CONF = "src-tauri/tauri.conf.json";

function usage(message) {
  if (message) {
    console.error(message);
  }
  console.error("Usage: yarn version:set 0.18.0");
  process.exit(1);
}

function parseVersion(raw) {
  const trimmed = raw.trim();
  const version = trimmed.startsWith("v") ? trimmed.slice(1) : trimmed;
  if (!SEMVER.test(version)) {
    usage(`Invalid version: ${raw}`);
  }
  return version;
}

function bumpCargo(relativePath, version) {
  const path = join(ROOT, relativePath);
  const text = readFileSync(path, "utf8");
  const next = text.replace(
    /(\[package\][\s\S]*?^version\s*=\s*")([^"]+)(")/m,
    `$1${version}$3`,
  );
  if (next === text && !text.includes(`version = "${version}"`)) {
    usage(`Could not update version in ${relativePath}`);
  }
  writeFileSync(path, next);
}

function bumpTauriConf(version) {
  const path = join(ROOT, TAURI_CONF);
  const text = readFileSync(path, "utf8");
  const next = text.replace(/("version"\s*:\s*")([^"]+)(")/, `$1${version}$3`);
  if (
    next === text &&
    !text.includes(`"version": "${version}"`)
  ) {
    usage(`Could not update version in ${TAURI_CONF}`);
  }
  writeFileSync(path, next);
}

const raw = process.argv[2];
if (!raw) {
  usage();
}

const version = parseVersion(raw);
for (const file of CARGO_FILES) {
  bumpCargo(file, version);
}
bumpTauriConf(version);
console.log(`Set version ${version} (tag v${version})`);
