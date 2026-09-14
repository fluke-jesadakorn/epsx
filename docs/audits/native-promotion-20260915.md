# Native release verification — 15 September 2026

Release source: `71c97f82a533434baa539c723aa95d1d35068bc6`.
Package: `20260915-full-71c97f82a`.
Manifest SHA-256: `728d38944322df3729bfba3f1ede44abda0ba25cf5db34394c5cdc4746640eb0`.

The package contains ten native binaries and the complete Frontend, Admin and
Pay hydration assets. All 521 manifest entries were verified. Migration files
match the currently installed native release; the explicit pending check found
no pending migrations. Development CSS reload code is absent from the packaged
web bundles. The only generated checkout difference is removal of four unused
Admin Tailwind utilities by the native Tailwind compiler.

Local validation passed: 2,329 workspace tests (44 ignored), strict workspace
Clippy, formatting, 33 contract tests, 11 development helper tests, frozen assets,
Rust-only, migration, authority and E2E evidence audits. The additional
Tailwind-only offline recovery regression, service-worker WASM compile check and
two bootstrap tests also passed.

Authenticated development pagination was verified at 10, 25, 50 and 100 rows,
including page two and reload persistence. Release-binary browser checks against
development upstreams verified 25/50 rows, page two showing 51–100, and retention
of 50 after reload. Frontend loaded one Tailwind stylesheet with no inline style
elements. About, Plans and Offline returned 200; Account preserved its sign-in
return URL. Admin's hydrated menu and Pay's developer-guide navigation worked.
These checks did not execute payments or account mutations.

## Protected promotion ancestry

The initial application PR was merged into development as `1cfc1b439`.
Staging (`8f6514bf5`) and production (`e72a32e0d`) retain historical environment
merge commits from May. Strict up-to-date protection therefore blocks a direct
forward promotion even though merging those histories produces the same source
tree. GitHub also rejects updating the protected development branch directly.

The promotion-history working branch merges both historical environment heads
before returning through a development PR. Its pre-documentation tree is
`205b6a4fb3015ce9fa15f527b3c4de8ebfce34ff`, identical to the tested release source.
This audit record is the only file change in that history-reconciliation PR.
Required checks and branch protection remain enabled for each forward promotion.

This records pre-deployment verification, not completed production deployment.
The existing Pay service returned 503 readiness before cutover because one
mainnet merchant scanner was behind; the other two mainnet merchant checkpoints
were healthy. Preserve those checkpoints and report chain readiness separately
from application HTTP readiness after deployment.
