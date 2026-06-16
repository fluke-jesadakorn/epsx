// specs/runner.spec.js — placeholder for future @playwright/test integration.
//
// Currently the harness runs a single parameterized Node script
// (scripts/capture.js) per the wave-23 T1 pattern. We keep this spec file
// as a marker for future migration to @playwright/test.
//
// To run with @playwright/test:
//   NODE_PATH=/Users/fluke/Desktop/Work/epsx/apps-old/frontend/node_modules \
//     npx playwright test specs/runner.spec.js
//
// The capture script handles the actual visit + click + screenshot loop;
// see scripts/capture.js for the implementation.
