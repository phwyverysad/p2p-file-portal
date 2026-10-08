import fs from 'fs';
import path from 'path';

const lockPath = path.resolve('src-tauri/Cargo.lock');
if (!fs.existsSync(lockPath)) {
  console.error('Cargo.lock not found at', lockPath);
  process.exit(1);
}

const content = fs.readFileSync(lockPath, 'utf8');

// Parse [[package]] blocks
const packages = [];
const blocks = content.split('[[package]]');

for (const block of blocks) {
  const nameMatch = block.match(/name\s*=\s*"([^"]+)"/);
  const versionMatch = block.match(/version\s*=\s*"([^"]+)"/);
  const sourceMatch = block.match(/source\s*=\s*"registry\+([^"]+)"/);

  if (nameMatch && versionMatch && sourceMatch) {
    packages.push({
      name: nameMatch[1],
      version: versionMatch[1],
    });
  }
}

console.log(`📦 Found ${packages.length} crates to fetch.`);

const userProfile = process.env.USERPROFILE || process.env.HOME;
const targetDirs = [
  path.join(userProfile, '.cargo', 'registry', 'cache', 'github.com-25cdd57fae9f0462'),
];

// Check if index.crates.io cache dir exists too
const registryDir = path.join(userProfile, '.cargo', 'registry', 'cache');
if (fs.existsSync(registryDir)) {
  for (const d of fs.readdirSync(registryDir)) {
    const full = path.join(registryDir, d);
    if (fs.statSync(full).isDirectory() && !targetDirs.includes(full)) {
      targetDirs.push(full);
    }
  }
}

for (const dir of targetDirs) {
  fs.mkdirSync(dir, { recursive: true });
}

let downloaded = 0;
let skipped = 0;
let failed = 0;

async function downloadCrate(pkg) {
  const fileName = `${pkg.name}-${pkg.version}.crate`;
  const primaryFile = path.join(targetDirs[0], fileName);

  if (fs.existsSync(primaryFile) && fs.statSync(primaryFile).size > 0) {
    skipped++;
    return;
  }

  const url = `https://static.crates.io/crates/${pkg.name}/${pkg.version}/download`;
  try {
    const res = await fetch(url);
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}`);
    }
    const buf = Buffer.from(await res.arrayBuffer());

    for (const dir of targetDirs) {
      fs.writeFileSync(path.join(dir, fileName), buf);
    }
    downloaded++;
    if (downloaded % 20 === 0 || downloaded === packages.length) {
      console.log(`⬇️ Progress: ${downloaded} downloaded, ${skipped} cached, total: ${packages.length}`);
    }
  } catch (err) {
    failed++;
    console.warn(`⚠️ Failed to download ${fileName}: ${err.message}`);
  }
}

// Concurrency pool
const CONCURRENCY = 12;
let idx = 0;

async function worker() {
  while (idx < packages.length) {
    const current = packages[idx++];
    await downloadCrate(current);
  }
}

console.log(`🚀 Starting parallel fetch with concurrency ${CONCURRENCY}...`);
const workers = Array.from({ length: CONCURRENCY }, () => worker());
await Promise.all(workers);

console.log(`\n🎉 Finished! Downloaded: ${downloaded}, Cached: ${skipped}, Failed: ${failed}`);
