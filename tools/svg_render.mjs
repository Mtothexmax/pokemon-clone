// Renders SVG files to PNG with headless Brave (the engine's Playwright): node tools/svg_render.mjs out_dir a.svg b.svg ...
// Each PNG has the SVG's width x height in pixels. Used by tools/make_signs.py.
import { createRequire } from "module";
import fs from "fs";
import path from "path";
import os from "os";

const require = createRequire("C:/Users/mtoth/AppData/Local/hz/engines/0.7.0.002/web/tests/package.json");
const { chromium } = require("playwright");
const browsers = [
  "C:/Program Files/BraveSoftware/Brave-Browser/Application/brave.exe",
  "C:/Program Files/Google/Chrome/Application/chrome.exe",
  "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
];
const exe = browsers.find((b) => fs.existsSync(b));
const [outDir, ...files] = process.argv.slice(2);
const dir = fs.mkdtempSync(path.join(os.tmpdir(), "svg-"));
const ctx = await chromium.launchPersistentContext(dir, { executablePath: exe, headless: true });
const page = ctx.pages()[0] || (await ctx.newPage());
for (const f of files) {
  const svg = fs.readFileSync(f, "utf8");
  const w = +/width="(\d+)"/.exec(svg)[1], h = +/height="(\d+)"/.exec(svg)[1];
  await page.setViewportSize({ width: w, height: h });
  await page.setContent(`<!doctype html><html><body style="margin:0">${svg}</body></html>`);
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: path.join(outDir, path.basename(f, ".svg") + ".png"), clip: { x: 0, y: 0, width: w, height: h } });
}
await ctx.close();
fs.rmSync(dir, { recursive: true, force: true });
console.log("rendered", files.length);
