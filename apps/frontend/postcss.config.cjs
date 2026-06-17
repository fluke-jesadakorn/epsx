// EPSX frontend BFF — PostCSS pipeline.
// Mirrors apps-old/frontend/postcss.config.cjs (Tailwind v4 + autoprefixer).
// build.rs runs `bun run build:css` to compile src/styles/index.css
// into public/dist/tailwind.css before cargo build.
module.exports = {
  plugins: {
    '@tailwindcss/postcss': {},
    autoprefixer: {},
  },
};