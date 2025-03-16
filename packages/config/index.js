const baseEslint = require('./eslint/base');
const prettierConfig = require('./prettier');
const tsconfig = require('./typescript/base.json');

module.exports = {
  eslint: {
    base: baseEslint,
  },
  prettier: prettierConfig,
  typescript: {
    base: tsconfig,
  },
  // Add paths for easy access in other packages
  paths: {
    eslint: {
      base: require.resolve('./eslint/base'),
    },
    prettier: require.resolve('./prettier'),
    typescript: {
      base: require.resolve('./typescript/base.json'),
    },
  },
};
