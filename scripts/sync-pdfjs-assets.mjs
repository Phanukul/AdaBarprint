import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const srcRoot = path.join(rootDir, "node_modules", "pdfjs-dist");
const dstRoot = path.join(rootDir, "public", "pdfjs");

const SUBDIRS = ["wasm", "iccs", "cmaps", "standard_fonts"];

export function syncPdfjsAssets() {
  if (!fs.existsSync(srcRoot)) {
    return;
  }
  for (const sub of SUBDIRS) {
    const src = path.join(srcRoot, sub);
    const dst = path.join(dstRoot, sub);
    if (!fs.existsSync(src)) continue;
    fs.mkdirSync(dst, { recursive: true });
    const srcEntries = fs.readdirSync(src);
    const dstEntries = fs.existsSync(dst) ? fs.readdirSync(dst) : [];
    if (dstEntries.length === srcEntries.length && srcEntries.length > 0) {
      continue;
    }
    fs.cpSync(src, dst, { recursive: true, force: true });
  }
}

const isCli =
  process.argv[1] &&
  path.resolve(process.argv[1]) === path.resolve(fileURLToPath(import.meta.url));
if (isCli) {
  syncPdfjsAssets();
}
