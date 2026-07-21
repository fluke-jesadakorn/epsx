import {
  createPrivateKey,
  createPublicKey,
  createSign,
  generateKeyPairSync,
} from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';

const command = process.argv[2];
const keyPath = process.argv[3];
const issuer = process.argv[4] ?? 'http://127.0.0.1:18082';
const kid = 'a8-admin-denial-ephemeral';

if (!keyPath || !['generate', 'serve', 'token'].includes(command)) {
  throw new Error(
    'usage: denial-auth-mock.mjs <generate|serve|token> <private-key-path> [issuer]'
  );
}

if (command === 'generate') {
  const { privateKey } = generateKeyPairSync('rsa', { modulusLength: 2048 });
  writeFileSync(keyPath, privateKey.export({ type: 'pkcs8', format: 'pem' }), {
    mode: 0o600,
  });
  process.exit(0);
}

const privateKey = createPrivateKey(readFileSync(keyPath));
const publicJwk = createPublicKey(privateKey).export({ format: 'jwk' });
const jwk = { ...publicJwk, alg: 'RS256', use: 'sig', kid };

function encodeJson(value) {
  return Buffer.from(JSON.stringify(value)).toString('base64url');
}

function accessToken() {
  const now = Math.floor(Date.now() / 1000);
  const header = encodeJson({ alg: 'RS256', typ: 'JWT', kid });
  const wallet = '0x00000000000000000000000000000000000000a8';
  const claims = encodeJson({
    iss: issuer,
    sub: wallet,
    aud: ['epsx-admin'],
    exp: now + 300,
    iat: now - 1,
    jti: `a8-admin-denial-${now}`,
    scope: 'openid permissions admin:dashboard:view',
    wallet_address: wallet,
    auth_method: 'web3_siwe',
    auth_time: now - 1,
  });
  const signingInput = `${header}.${claims}`;
  const signature = createSign('RSA-SHA256')
    .update(signingInput)
    .end()
    .sign(privateKey)
    .toString('base64url');
  return `${signingInput}.${signature}`;
}

if (command === 'token') {
  process.stdout.write(accessToken());
  process.exit(0);
}

const parsedIssuer = new URL(issuer);
if (
  parsedIssuer.protocol !== 'http:' ||
  parsedIssuer.hostname !== '127.0.0.1' ||
  parsedIssuer.username !== '' ||
  parsedIssuer.password !== '' ||
  parsedIssuer.pathname !== '/' ||
  parsedIssuer.search !== '' ||
  parsedIssuer.hash !== ''
) {
  throw new Error(
    'the A8 denial fixture requires a plain 127.0.0.1 HTTP origin'
  );
}
const port = Number(parsedIssuer.port);
if (!Number.isInteger(port) || port < 1024 || port > 65535) {
  throw new Error(
    'the A8 denial fixture requires an explicit unprivileged port'
  );
}

const server = createServer((request, response) => {
  if (request.url === '/.well-known/jwks.json') {
    response.writeHead(200, { 'content-type': 'application/json' });
    response.end(JSON.stringify({ keys: [jwk] }));
    return;
  }
  if (request.url === '/healthz') {
    response.writeHead(204);
    response.end();
    return;
  }
  if (request.url === '/api/auth/web3/logout' && request.method === 'DELETE') {
    let body = '';
    request.on('data', chunk => {
      body += chunk;
      if (body.length > 16 * 1024) request.destroy();
    });
    request.on('end', () => {
      response.writeHead(200, { 'content-type': 'application/json' });
      response.end(JSON.stringify({ success: true }));
    });
    return;
  }
  response.writeHead(404, { 'content-type': 'application/json' });
  response.end(JSON.stringify({ error: 'not_found' }));
});
server.listen(port, '127.0.0.1');
