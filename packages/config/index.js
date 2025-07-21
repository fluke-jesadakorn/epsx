const baseEslint = require('./eslint/base');
const prettierConfig = require('./prettier');
const tsconfig = require('./typescript/base.json');
const postcssConfig = require('./postcss');

module.exports = {
  eslint: {
    base: baseEslint,
  },
  prettier: prettierConfig,
  postcss: postcssConfig,
  typescript: {
    base: tsconfig,
  },
  // Add paths for easy access in other packages
  paths: {
    eslint: {
      base: require.resolve('./eslint/base'),
    },
    prettier: require.resolve('./prettier'),
    postcss: require.resolve('./postcss'),
    typescript: {
      base: require.resolve('./typescript/base.json'),
    },
  },
};
