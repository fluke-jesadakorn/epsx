// Wave 21 dev-old.sh stub for `remark-gfm`.
//
// The OLD frontend imports this as a remark plugin for react-markdown.
// Since react-markdown is also stubbed (no real AST traversal), this
// plugin is a no-op passthrough — the markdown is rendered as raw text
// by the react-markdown stub. See scripts/old-stubs/react-markdown/
// for the trade-off rationale.

module.exports = function remarkGfmStub() {
  return (tree) => tree;
};
module.exports.default = module.exports;
