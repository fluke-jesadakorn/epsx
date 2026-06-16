// specs/routes.spec.js — placeholder parameterized spec.
// The actual capture/diff runs from capture-*.sh + diff.sh, which wrap
// capture.js. This file documents the per-route contract and is the seed
// for any future @playwright/test integration.

const routes = require("../scripts/routes.json");

module.exports = {
  routes: routes.routes,
  note:
    "captured by capture.js — see capture-prod.sh / capture-dev.sh. " +
    "Each route: navigate, hydrate 4s, screenshot, click all <a>/<button>/" +
    "[role=button]/details/summary/select/input/textarea/[tabindex], " +
    "record interactions.jsonl, console.log, network.jsonl, redirects.log.",
};
