// EPSX admin BFF — PostCSS pipeline.
// Mirrors apps-old/admin-frontend/postcss.config.cjs (Tailwind v4 + autoprefixer).
module.exports = {
  plugins: {
    '@tailwindcss/postcss': {},
    autoprefixer: {},
  },
};