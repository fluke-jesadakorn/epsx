// Wave 21 dev-old.sh stub for micromark.
//
// Why: `apps-old/admin-frontend/components/news/news-editor.tsx` does
//   const { micromark } = await import('micromark');
//   const { gfm, gfmHtml } = await import('micromark-extension-gfm');
//   return micromark(md, { extensions: [gfm()], htmlExtensions: [gfmHtml()] });
// but neither `micromark` nor `micromark-extension-gfm` are declared in
// the OLD app's package.json. They were hoisted in the OLD monorepo.
//
// In dev mode, webpack hard-fails on `Module not found: Can't resolve 'micromark'`,
// which 500s the entire /news/* route.
//
// Stub: a no-op `micromark(md, opts) => md` that returns the raw markdown
// wrapped in a <pre> tag, so the news editor page renders. The visual diff
// is done in the static structure of the page, not the markdown render.

function micromark(md) {
  // Return the raw markdown wrapped in a <pre>. The dev-only news editor
  // page will not be used for actual publishing in this dev loop.
  const escaped = String(md ?? '').replace(/[<>&]/g, (c) =>
    c === '<' ? '&lt;' : c === '>' ? '&gt;' : '&amp;'
  );
  return `<pre>${escaped}</pre>`;
}

module.exports = { micromark, gfm: () => ({}), gfmHtml: () => ({}) };
module.exports.default = module.exports;
