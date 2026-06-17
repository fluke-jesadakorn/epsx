// Wave 21 dev-old.sh stub for @tailwindcss/typography.
//
// Why: the OLD admin-frontend and OLD frontend both reference
// `@plugin "@tailwindcss/typography";` in styles/index.css, but
// the plugin is NOT in their package.json (it was hoisted in the
// OLD monorepo). The plugin's full dep tree is
// {lodash.castarray, lodash.isplainobject, lodash.merge,
// postcss-selector-parser}, none of which the OLD apps declare.
//
// In dev mode with Tailwind v4 this causes a hard compile error
// on every page request. We stub it as a no-op plugin so the dev
// server boots and pages render. The `prose` / `prose-sm` utility
// classes will produce no styles in this mode — that's the
// tradeoff for getting a working dev loop on the OLD apps.
//
// To get the real plugin back, install it via pnpm in the OLD
// app's node_modules (would require touching package.json, which
// the wave-21 brief forbids).
module.exports = function typographyStub() {
  return { handler() {} };
};
