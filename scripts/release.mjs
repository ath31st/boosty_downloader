import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const TAURI_CONF = join(ROOT, "src-tauri/tauri.conf.json");

function fail(message) {
  console.error(message);
  process.exit(1);
}

function git(args, { capture = false } = {}) {
  const result = spawnSync("git", args, {
    cwd: ROOT,
    encoding: "utf8",
    stdio: capture ? ["ignore", "pipe", "pipe"] : "inherit",
  });
  if (result.status !== 0) {
    if (capture && result.stderr) {
      process.stderr.write(result.stderr);
    }
    process.exit(result.status ?? 1);
  }
  return capture ? result.stdout.trim() : "";
}

function gitOk(args) {
  const result = spawnSync("git", args, {
    cwd: ROOT,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  return result.status === 0;
}

const conf = JSON.parse(readFileSync(TAURI_CONF, "utf8"));
const version = typeof conf.version === "string" ? conf.version.trim() : "";
if (!version) {
  fail("No version in src-tauri/tauri.conf.json");
}

const tag = `v${version}`;
const branch = git(["branch", "--show-current"], { capture: true });
if (branch !== "master") {
  fail(`Release from master, current branch is ${branch || "(detached)"}`);
}

const dirty = git(["status", "--porcelain"], { capture: true });
if (dirty) {
  fail("Working tree is dirty. Commit or stash first.");
}

if (gitOk(["rev-parse", "-q", "--verify", `refs/tags/${tag}`])) {
  fail(`Tag ${tag} already exists locally`);
}

console.log(`Releasing ${tag}`);
git(["push", "origin", "master"]);
git(["tag", tag]);
git(["push", "origin", tag]);
console.log(`Pushed ${tag}. GitHub Actions will attach the binaries to the release.`);
