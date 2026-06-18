// Copies the game data + class icons from src/ into public/ so Vite bundles them
// into dist (and thus into the exe). These copied folders are gitignored and
// regenerated here, so a fresh clone builds correctly without committing
// generated files. Runs automatically via the predev / prebuild npm hooks.
const fs = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..");
const jobs = [
  ["src/data/i18n", "public/i18n"],
  ["src/assets", "public/assets"],
];

function copyDir(from, to) {
  fs.mkdirSync(to, { recursive: true });
  fs.cpSync(from, to, { recursive: true });
}

for (const [from, to] of jobs) {
  const src = path.join(root, from);
  const dst = path.join(root, to);
  if (!fs.existsSync(src)) {
    console.warn(`[sync-assets] skip missing ${from}`);
    continue;
  }
  copyDir(src, dst);
  console.log(`[sync-assets] ${from} -> ${to}`);
}

// JSON data files (skill_icons.json, dot_skill_ids.json) -> public/data
const dataOut = path.join(root, "public/data");
fs.mkdirSync(dataOut, { recursive: true });
const dataDir = path.join(root, "src/data");
for (const file of fs.readdirSync(dataDir)) {
  if (file.endsWith(".json")) {
    fs.copyFileSync(path.join(dataDir, file), path.join(dataOut, file));
    console.log(`[sync-assets] src/data/${file} -> public/data/${file}`);
  }
}
