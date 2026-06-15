// Wave 21 dev-old.sh stub for `react-markdown`.
//
// Why: the OLD frontend's `components/news/news-detail.tsx` imports
// from `react-markdown` and `remark-gfm`, but neither is declared
// in the OLD app's package.json. They were hoisted in the original
// monorepo's root node_modules.
//
// The OLD app's `app/news/[slug]/page.tsx` uses <MarkdownAsync> to
// render the article body. With this stub, the article body renders
// as a plain <pre> containing the raw markdown source — readable
// enough for a visual diff, but NOT styled.
//
// In dev mode (next dev --webpack) the missing-import is a HARD
// module-not-found error, not a warning, so without this stub the
// page returns 500.
//
// To get the real react-markdown back, install it in the OLD app:
//   pnpm add -F apps-old-frontend react-markdown remark-gfm
// (would require modifying package.json + pnpm-lock.yaml, which the
// wave-21 brief forbids).

const React = require('react');

function MarkdownAsync({ children, ...props }) {
  // Render the raw markdown as a <pre> so the page is at least
  // inspectable. Strip GFM/remark-specific options we don't support.
  const text = typeof children === 'string'
    ? children
    : Array.isArray(children)
      ? children.join('')
      : String(children ?? '');
  return React.createElement('pre', {
    'data-react-markdown-stub': 'true',
    className: 'react-markdown-stub whitespace-pre-wrap font-mono text-sm leading-relaxed',
    ...props
  }, text);
}

module.exports = MarkdownAsync;
module.exports.default = MarkdownAsync;
