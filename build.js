import * as esbuild from 'esbuild';
import esbuildSvelte from 'esbuild-svelte';
import sveltePreprocess from 'svelte-preprocess';
import fs from 'fs';
import path from 'path';
import postcss from 'postcss';
import tailwindcss from 'tailwindcss';
import autoprefixer from 'autoprefixer';

const isWatch = process.argv.includes('--watch');
const distDir = path.resolve('dist');

if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

async function buildCss() {
  const cssInputPath = path.resolve('src/app.css');
  if (fs.existsSync(cssInputPath)) {
    const cssContent = fs.readFileSync(cssInputPath, 'utf8');
    const result = await postcss([tailwindcss, autoprefixer]).process(cssContent, {
      from: cssInputPath,
      to: path.join(distDir, 'bundle.css')
    });
    fs.writeFileSync(path.join(distDir, 'bundle.css'), result.css);
  }
}

function copyPublicAssets() {
  const publicDir = path.resolve('public');
  if (fs.existsSync(publicDir)) {
    const entries = fs.readdirSync(publicDir, { withFileTypes: true });
    for (const entry of entries) {
      const srcPath = path.join(publicDir, entry.name);
      const destPath = path.join(distDir, entry.name);
      if (entry.isFile()) {
        fs.copyFileSync(srcPath, destPath);
      }
    }
  }
}

async function run() {
  copyPublicAssets();
  await buildCss();

  const ctx = await esbuild.context({
    entryPoints: ['src/main.ts'],
    bundle: true,
    outdir: 'dist',
    entryNames: 'bundle',
    format: 'iife',
    platform: 'browser',
    sourcemap: true,
    minify: !isWatch,
    plugins: [
      esbuildSvelte({
        preprocess: sveltePreprocess({
          typescript: true
        }),
      })
    ],
    loader: {
      '.svg': 'text',
      '.png': 'dataurl',
    }
  });

  if (isWatch) {
    console.log('⚡ Watching for changes...');
    await ctx.watch();
  } else {
    await ctx.rebuild();
    await ctx.dispose();
    console.log('✅ Build completed successfully into dist/');
  }
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
