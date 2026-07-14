#!/usr/bin/env node

const path = require("path");
const { spawn } = require("child_process");

const binaryName = process.platform === "win32" ? "sks-native.exe" : "sks-native";
const binaryPath = path.join(__dirname, binaryName);
const child = spawn(binaryPath, process.argv.slice(2), {
  argv0: "sks",
  stdio: "inherit",
});

child.on("error", (error) => {
  console.error(`无法启动 SkillsMP CLI: ${error.message}`);
  console.error("请重新安装: npm install -g skillsmp-search");
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }

  process.exit(code ?? 1);
});
