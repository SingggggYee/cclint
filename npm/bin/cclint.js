#!/usr/bin/env node
// Shim: forward all args to the platform binary downloaded by install.js.

"use strict";

const { spawnSync } = require("node:child_process");
const { join } = require("node:path");
const { existsSync } = require("node:fs");

const bin = join(__dirname, "..", "vendor", "cclint");

if (!existsSync(bin)) {
  console.error("cclint: binary missing. Reinstall the package (npm rebuild cclint)");
  console.error("or install via cargo: cargo install cclint");
  process.exit(1);
}

const result = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
process.exit(result.status ?? 1);
