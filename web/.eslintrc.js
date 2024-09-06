const path = require('path');

module.exports = {
  env: {
    browser: true,
    es2020: true,
  },
  extends: [
    'plugin:react/recommended',
    'eslint:recommended',
    'plugin:@typescript-eslint/eslint-recommended',
    'plugin:@typescript-eslint/recommended'
  ],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaFeatures: {
      jsx: true,
    },
    ecmaVersion: 11,
    sourceType: 'module',
    project: path.join(__dirname, "tsconfig.json"),
  },
  settings: {
    react: {
      version: 'detect',
    }
  },
  plugins: [
    'react',
    '@typescript-eslint',
    'deprecation',
  ],
  rules: {
    "deprecation/deprecation": "warn",
    "semi": ["error", "always"],
    "no-trailing-spaces": ["error"],
    "react/prop-types": "off"
  },
  overrides: [
    {
      // enable the rule specifically for TypeScript files
      "files": ["*.js", "*.jsx"],
      "rules": {
        "@typescript-eslint/explicit-function-return-type": "off",
        "@typescript-eslint/explicit-module-boundary-types": "off",
      },
    }
  ],
  ignorePatterns: [
    'src/protos/**'
  ]
};
