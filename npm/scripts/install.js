#!/usr/bin/env node
/**
 * 安装脚本：根据当前平台下载对应的预编译二进制文件
 *
 * 工作流程：
 * 1. 检测当前操作系统和 CPU 架构
 * 2. 从 GitHub Releases 下载对应的二进制文件
 * 3. 保存到 npm 包的 bin 目录
 */

const fs = require("fs");
const path = require("path");
const https = require("https");
const { pipeline } = require("stream");
const { promisify } = require("util");

// 配置
const PACKAGE_NAME = "sks";
const GITHUB_REPO = "geraldpeng6/skillsmp";
const MAX_REDIRECTS = 5;
const pipelineAsync = promisify(pipeline);

// 平台映射：Node.js 平台名 -> 二进制文件名后缀
const PLATFORM_MAP = {
  "darwin-x64": "darwin-x86_64",
  "darwin-arm64": "darwin-aarch64",
  "linux-x64": "linux-x86_64",
  "win32-x64": "windows-x86_64.exe",
};

/**
 * 获取当前平台的二进制文件名
 */
function getBinaryName() {
  const platform = process.platform;
  const arch = process.arch;
  const key = `${platform}-${arch}`;

  const suffix = PLATFORM_MAP[key];
  if (!suffix) {
    console.error(`❌ 不支持的平台: ${key}`);
    console.error(`   支持的平台: ${Object.keys(PLATFORM_MAP).join(", ")}`);
    process.exit(1);
  }

  return `${PACKAGE_NAME}-${suffix}`;
}

/**
 * 获取下载 URL（始终使用最新版本）
 */
function getDownloadUrl(binaryName) {
  // 使用 latest 而非固定版本，npm 版本与二进制版本解耦
  return `https://github.com/${GITHUB_REPO}/releases/latest/download/${binaryName}`;
}

/**
 * 下载文件
 */
async function download(url, dest, redirectCount = 0) {
  const response = await new Promise((resolve, reject) => {
    https.get(url, resolve).on("error", reject);
  });

  const isRedirect = [301, 302, 303, 307, 308].includes(response.statusCode);
  if (isRedirect && response.headers.location) {
    response.resume();

    if (redirectCount >= MAX_REDIRECTS) {
      throw new Error("下载失败: 重定向次数过多");
    }

    const nextUrl = new URL(response.headers.location, url).toString();
    return download(nextUrl, dest, redirectCount + 1);
  }

  if (response.statusCode !== 200) {
    response.resume();
    throw new Error(`下载失败: HTTP ${response.statusCode}`);
  }

  const tempPath = `${dest}.download`;

  try {
    await pipelineAsync(response, fs.createWriteStream(tempPath));

    if (process.platform === "win32" && fs.existsSync(dest)) {
      fs.unlinkSync(dest);
    }

    fs.renameSync(tempPath, dest);
  } catch (error) {
    if (fs.existsSync(tempPath)) {
      fs.unlinkSync(tempPath);
    }
    throw error;
  }
}

/**
 * 主函数
 */
async function main() {
  const binDir = path.join(__dirname, "..", "bin");
  const binaryName = getBinaryName();
  const isWindows = process.platform === "win32";
  const destName = isWindows ? `${PACKAGE_NAME}-native.exe` : `${PACKAGE_NAME}-native`;
  const destPath = path.join(binDir, destName);

  // 创建 bin 目录
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  // 下载二进制文件
  const url = getDownloadUrl(binaryName);

  try {
    console.log(`📥 下载: ${url}`);
    await download(url, destPath);

    // 设置可执行权限（非 Windows）
    if (!isWindows) {
      fs.chmodSync(destPath, 0o755);
    }

    console.log(`✅ 安装成功!`);
    console.log("");
    console.log("使用方法:");
    console.log("  sks <关键词>          搜索 AI Skills");
    console.log("  sks python --limit 5  搜索并限制返回数量");
    console.log("  sks --help            查看帮助");
  } catch (err) {
    console.error(`❌ 安装失败: ${err.message}`);
    console.error("");
    console.error("可能的原因:");
    console.error("  1. 网络问题，请检查网络连接");
    console.error("  2. 该版本尚未发布预编译二进制");
    console.error("");
    console.error("手动下载:");
    console.error(`  ${url}`);
    process.exit(1);
  }
}

main();
