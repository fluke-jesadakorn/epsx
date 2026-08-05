import { createHash } from 'node:crypto';
import {
  mkdir,
  readFile,
  readdir,
  rename,
  stat,
  writeFile,
} from 'node:fs/promises';
import { dirname, relative, resolve } from 'node:path';

export async function ensureDirectory(path: string): Promise<void> {
  await mkdir(path, { recursive: true });
}

export async function writeJson(path: string, value: unknown): Promise<void> {
  await ensureDirectory(dirname(path));
  const temporary = `${path}.tmp-${process.pid}`;
  await writeFile(temporary, `${JSON.stringify(value, null, 2)}\n`, 'utf8');
  await rename(temporary, path);
}

export async function readJson<T>(path: string): Promise<T> {
  return JSON.parse(await readFile(path, 'utf8')) as T;
}

export function sha256(value: string | Buffer): string {
  return createHash('sha256').update(value).digest('hex');
}

export async function sha256File(path: string): Promise<string> {
  return sha256(await readFile(path));
}

export function stableJson(value: unknown): string {
  return JSON.stringify(sortValue(value));
}

function sortValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(sortValue);
  }
  if (value !== null && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .sort(([left], [right]) => left.localeCompare(right))
        .map(([key, child]) => [key, sortValue(child)])
    );
  }
  return value;
}

export function slugify(value: string): string {
  const slug = value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '-')
    .replace(/^-+|-+$/g, '');
  if (slug === '' || slug.includes('..')) {
    throw new Error(`unsafe artifact slug: ${value}`);
  }
  return slug;
}

export async function listFiles(root: string): Promise<string[]> {
  const output: string[] = [];
  async function visit(directory: string): Promise<void> {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const path = resolve(directory, entry.name);
      if (entry.isDirectory()) {
        await visit(path);
      } else if (entry.isFile()) {
        output.push(path);
      }
    }
  }
  try {
    await visit(root);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
      throw error;
    }
  }
  return output.sort();
}

export async function artifactManifest(
  root: string,
  excludedRelativePaths: string[] = []
): Promise<Array<{ path: string; bytes: number; sha256: string }>> {
  const excluded = new Set(excludedRelativePaths);
  const entries = [];
  for (const path of await listFiles(root)) {
    const artifactPath = relative(root, path).replaceAll('\\', '/');
    if (excluded.has(artifactPath)) {
      continue;
    }
    entries.push({
      path: artifactPath,
      bytes: (await stat(path)).size,
      sha256: await sha256File(path),
    });
  }
  return entries;
}
