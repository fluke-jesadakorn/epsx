# EPSX Editorial — frontend presentation refresh

Implemented 8 September 2026 against the existing working tree. Frontend now uses
an ivory/ink/forest palette, with a matching dark theme and Georgia for Home and
news headlines. Company symbols, navigation, forms and numeric data remain sans
serif. No layout, copy, route, API, authorization or subscription changes were
made. The EPSX logo and saved theme preference are retained.

## Review

- Real-backend local preview: <http://127.0.0.1:3321/>
- Company rankings: <http://127.0.0.1:3321/analytics>
- Before/after gallery, seven pages in both themes: <http://127.0.0.1:3326/>
- Offline gallery: `target/editorial-refresh/review/index.html`.

The gallery uses the same isolated fixture data before and after, explicitly
marked **Preview data** in the screenshots. The product preview on 3321 uses the
existing native backend on 8080 and notification service on 8106. Existing
development servers and production routes were not replaced.

## Implementation

- `apps/frontend/public/enterprise.css`: scoped theme tokens, semantic legacy
  utility aliases, warm surfaces/shadows, editorial headings, ink company
  symbols, forest actions, readable input borders and error/pending states.
- `apps/frontend/src/enterprise.rs`: stylesheet cache version `editorial-1`.
- Interaction transitions run on hover/focus rather than initial theme
  application, preventing a flash of light controls when restoring dark mode.
- Existing non-presentation CSS declarations remain identical across 764 rules.
  No shared component source, Admin, Pay, backend, dependency or font asset was
  changed. The new design uses system fonts and the existing native BFF.

## Validation

| Check | Result |
| --- | --- |
| Native frontend build, locked | Passed |
| UI, frontend and browser-runtime Rust tests | 977 passed: 840 + 124 + 13 |
| Workspace format check | Passed |
| Frontend Clippy, all targets, locked, warnings denied | Passed |
| Asset verification | Passed, five required assets |
| Existing SSR route/copy audit | 39 routes passed |
| Existing Explore state audit | Eight cases passed |
| Admin and Pay HTTP isolation checks | Passed; frontend stylesheet absent |
| Browser matrix | 39 routes × 375/768/1024/1440px × light/dark = 312 passed |
| Browser state captures | Empty/unavailable/restricted × two widths × two themes = 12 passed |
| Measured text contrast | 17,792 samples; minimum 4.55:1 |

The browser matrix checks document overflow, one main landmark, one h1, the
versioned stylesheet, the selected theme and rendered text contrast. Home,
rankings, Saved companies, plans, account, news, sign-in and error-state images
were also visually inspected. A real-backend news article was inspected in
addition to the fixture route matrix.

Browser workflows verified country filtering, pagination preserving that filter,
Save confirmation and persistence after reload, moving a saved company into a
group, and theme persistence. An injected fixture failure verified the pending
state, an accessible error, and preservation of the previously confirmed Saved
state. Keyboard Enter opens the mobile menu with a visible focus outline;
Escape closes it and returns focus to the trigger. Sign-in retains the complete
filtered return URL. Test mutations were confined to the separate fixture
service, and fixture state was restored afterward.

Contrast measurements cover rendered text on compositable CSS backgrounds;
background images, SVG content, replaced progress fallback text and disabled
controls are excluded. This is not a complete accessibility certification.
Reduced-motion suppression was verified in the stylesheet; media emulation was
not available in this browser surface. A real external-wallet signature, payment
or production mutation was not performed. Native WebDriver E2E was not run;
the browser checks above are recorded separately.

## Evidence and reproduction

`target/editorial-refresh/` contains the pre-change CSS/shell and native binary,
the exact task-only `source.patch`, 312 matrix captures with `audit.json`, the
before/after gallery, state captures, workflow evidence, SSR audit results and
build/test/lint logs. The inspection function is in `measure.js`.

Rebuild with `cargo build -p epsx-frontend --bin bff-frontend --locked`. The local
`restart_preview.py` helper restarts only this task's 3321/3324 processes and uses
the same development service URLs. Temporary QA used an independent authority
on 48181 and adapter on 48182, with public/authenticated frontend ports 3324/3325;
it did not reconfigure the existing 3300 product preview or shared 48081 fixture.
Temporary QA and baseline processes were stopped after verification; only the
real-backend preview on 3321 and the screenshot gallery on 3326 remain running.

The change is local only. Production deployment remains a separate operation.
