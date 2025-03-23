import { readFile } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const configPath = resolve(__dirname, '../dist/eslint/base.js');

const run = async () => {
  const mod = await import(configPath);
  console.log(JSON.stringify(mod.default, null, 2));
};

run().catch(console.error);
