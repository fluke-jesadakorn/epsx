#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');
const { spawn } = require('node:child_process');

const ENVIRONMENT_ALIASES = {
  dev: 'development',
  development: 'development',
  local: 'development',
  preview: 'development',
  test: 'development',
  stage: 'staging',
  staging: 'staging',
  prod: 'production',
  production: 'production',
  main: 'production',
  master: 'production',
};

function normalizeEnvironmentName(value) {
  if (value === undefined || value === null) {
    return undefined;
  }

  const normalized = String(value).trim().toLowerCase();
  if (normalized === '') {
    return undefined;
  }

  return ENVIRONMENT_ALIASES[normalized];
}

function resolveEnvironmentName(env = process.env) {
  const explicit = [
    env.DEPLOYMENT_ENV,
    env.NEXT_PUBLIC_DEPLOYMENT_ENV,
    env.APP_ENV,
    env.ENV,
    env.EPSX_ENV,
    env.RUST_ENV,
    env.NODE_ENV,
  ]
    .map(normalizeEnvironmentName)
    .find(Boolean);

  return explicit ?? 'development';
}

function getRootDir() {
  return path.resolve(__dirname, '..', '..');
}

function resolveRootEnvFile(rootEnvFile) {
  if (rootEnvFile === undefined || rootEnvFile === null || rootEnvFile === '') {
    return undefined;
  }

  return path.isAbsolute(rootEnvFile)
    ? rootEnvFile
    : path.resolve(process.cwd(), rootEnvFile);
}

function getEnvFiles({
  rootDir = getRootDir(),
  envName = resolveEnvironmentName(),
  rootEnvFile = process.env.ROOT_ENV_FILE,
} = {}) {
  const explicitRootEnvFile = resolveRootEnvFile(rootEnvFile);
  if (explicitRootEnvFile !== undefined) {
    return [explicitRootEnvFile];
  }

  return [
    path.join(rootDir, '.env'),
    path.join(rootDir, `.env.${envName}`),
    path.join(rootDir, '.env.local'),
    path.join(rootDir, `.env.${envName}.local`),
  ];
}

function stripWrappingQuotes(value) {
  if (value.length < 2) {
    return value;
  }

  const startsWithSingle = value.startsWith('\'') && value.endsWith('\'');
  const startsWithDouble = value.startsWith('"') && value.endsWith('"');
  if (!startsWithSingle && !startsWithDouble) {
    return value;
  }

  const inner = value.slice(1, -1);
  if (startsWithDouble) {
    return inner
      .replace(/\\n/g, '\n')
      .replace(/\\r/g, '\r')
      .replace(/\\t/g, '\t')
      .replace(/\\"/g, '"')
      .replace(/\\\\/g, '\\');
  }

  return inner;
}

function parseEnvContent(content) {
  const parsed = {};

  for (const line of content.split(/\r?\n/u)) {
    const trimmed = line.trim();
    if (trimmed === '' || trimmed.startsWith('#')) {
      continue;
    }

    const match = line.match(/^\s*(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)\s*$/u);
    if (!match) {
      continue;
    }

    const [, key, rawValue] = match;
    let value = rawValue.trim();

    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith('\'') && value.endsWith('\''))
    ) {
      value = stripWrappingQuotes(value);
    } else {
      value = value.replace(/\s+#.*$/u, '').trim();
    }

    parsed[key] = value;
  }

  return parsed;
}

function parseEnvFile(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  return parseEnvContent(content);
}

function buildRootEnv(baseEnv = process.env) {
  const rootDir = getRootDir();
  const envName = resolveEnvironmentName(baseEnv);
  const envFiles = getEnvFiles({
    rootDir,
    envName,
    rootEnvFile: baseEnv.ROOT_ENV_FILE,
  });

  const fileEnv = {};
  const loadedFiles = [];

  for (const filePath of envFiles) {
    if (!fs.existsSync(filePath)) {
      continue;
    }

    Object.assign(fileEnv, parseEnvFile(filePath));
    loadedFiles.push(filePath);
  }

  const mergedEnv = {
    ...fileEnv,
    ...baseEnv,
  };

  const derivedEnv = {};

  if (mergedEnv.ENV === undefined || mergedEnv.ENV === '') {
    mergedEnv.ENV = envName;
    derivedEnv.ENV = envName;
  }

  if (mergedEnv.DEPLOYMENT_ENV === undefined || mergedEnv.DEPLOYMENT_ENV === '') {
    mergedEnv.DEPLOYMENT_ENV = envName;
    derivedEnv.DEPLOYMENT_ENV = envName;
  }

  if (mergedEnv.RUST_ENV === undefined || mergedEnv.RUST_ENV === '') {
    mergedEnv.RUST_ENV = envName;
    derivedEnv.RUST_ENV = envName;
  }

  if (mergedEnv.NODE_ENV === undefined || mergedEnv.NODE_ENV === '') {
    const nodeEnv = envName === 'development' ? 'development' : 'production';
    mergedEnv.NODE_ENV = nodeEnv;
    derivedEnv.NODE_ENV = nodeEnv;
  }

  if (mergedEnv.EPSX_ENV === undefined || mergedEnv.EPSX_ENV === '') {
    mergedEnv.EPSX_ENV = envName;
    derivedEnv.EPSX_ENV = envName;
  }

  if (
    loadedFiles.length > 0 &&
    (mergedEnv.EPSX_ROOT_ENV_FILES === undefined || mergedEnv.EPSX_ROOT_ENV_FILES === '')
  ) {
    mergedEnv.EPSX_ROOT_ENV_FILES = loadedFiles.join(path.delimiter);
    derivedEnv.EPSX_ROOT_ENV_FILES = mergedEnv.EPSX_ROOT_ENV_FILES;
  }

  return {
    envName,
    rootDir,
    envFiles,
    loadedFiles,
    fileEnv,
    derivedEnv,
    mergedEnv,
  };
}

function shellQuote(value) {
  return `'${String(value).replace(/'/gu, `'\"'\"'`)}'`;
}

function printShellExports() {
  const { fileEnv, derivedEnv, mergedEnv } = buildRootEnv(process.env);
  const keys = new Set([...Object.keys(fileEnv), ...Object.keys(derivedEnv)]);
  const lines = [];

  for (const key of Array.from(keys).sort()) {
    if (process.env[key] === mergedEnv[key]) {
      continue;
    }

    lines.push(`export ${key}=${shellQuote(mergedEnv[key] ?? '')}`);
  }

  if (lines.length > 0) {
    process.stdout.write(`${lines.join('\n')}\n`);
  }
}

function execWithMergedEnv(args) {
  if (args.length === 0) {
    console.error('Usage: root-env.js --exec <command> [args...]');
    process.exit(1);
  }

  const { mergedEnv } = buildRootEnv(process.env);
  const child = spawn(args[0], args.slice(1), {
    stdio: 'inherit',
    env: mergedEnv,
  });

  child.on('error', (error) => {
    console.error(error.message);
    process.exit(1);
  });

  child.on('exit', (code, signal) => {
    if (signal !== null) {
      process.kill(process.pid, signal);
      return;
    }

    process.exit(code ?? 1);
  });
}

if (require.main === module) {
  const args = process.argv.slice(2);

  if (args[0] === '--print-shell') {
    printShellExports();
  } else if (args[0] === '--exec') {
    execWithMergedEnv(args.slice(1));
  } else {
    execWithMergedEnv(args);
  }
}

module.exports = {
  buildRootEnv,
  getEnvFiles,
  normalizeEnvironmentName,
  parseEnvContent,
  parseEnvFile,
  resolveEnvironmentName,
};
