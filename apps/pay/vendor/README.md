# WalletConnect transport exception

EPSX Pay loads this pinned, self-hosted bundle only when the buyer chooses
WalletConnect. Dioxus/Rust owns the checkout UI, invoice, network selection,
transaction request and fulfillment; `bridge.mjs` adapts SDK lifecycle only.

Source: official `@walletconnect/ethereum-provider` 2.24.0 registry archive.
`manifest.json` records source, SDK/bridge checksums and browser SRI.
`dependencies.lock.json` preserves the exact dependency tree and registry integrity.
The official package license and bundled third-party notices are included.

The checked-in browser bundle was produced from `bridge.mjs` using esbuild
0.25.12 (browser platform, IIFE, minification, linked legal comments), with the
locked packages installed in a temporary tooling directory. Updating it requires
regenerating the bundle and notices, reviewing the dependency lock, and updating
all checksums/SRI in the manifest. No JavaScript toolchain is needed to build or
run the native EPSX release.

`cargo xtask audit no-node --strict` allows exactly the SDK bundle and bridge;
it verifies their hashes plus the lock and licenses. Other scripts remain barred.
