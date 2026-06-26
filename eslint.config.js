import globals from "globals";
import eslintReact from "@eslint-react/eslint-plugin";
import pluginReactHooks from "eslint-plugin-react-hooks";
import pluginTs from "@typescript-eslint/eslint-plugin";
import parserTs from "@typescript-eslint/parser";

const tsBase = {
  languageOptions: {
    parser: parserTs,
    parserOptions: {
      ecmaFeatures: { jsx: true },
      project: "./tsconfig.json",
    },
  },
  plugins: {
    "@typescript-eslint": pluginTs,
    "react-hooks": pluginReactHooks,
    "@eslint-react": eslintReact,
  },
  rules: {
    ...pluginTs.configs.recommended.rules,
    ...pluginReactHooks.configs.recommended.rules,
    "no-unused-vars": "off",
    "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
    "no-console": ["warn", { allow: ["warn", "error"] }],
    // Downgraded: pre-existing codebase has legitimate `any` usage at API boundaries
    "@typescript-eslint/no-explicit-any": "warn",
    "@eslint-react/no-direct-mutation-state": "error",
    "@eslint-react/no-missing-key": "error",
    // Off: many pre-existing list renders use index keys where stable IDs aren't available
    "@eslint-react/no-array-index-key": "off",
    // react-hooks v7 new rules – overly strict for this project's data-fetching pattern
    "react-hooks/set-state-in-effect": "off",
  },
};

export default [
  {
    ignores: ["dist/", "node_modules/", "src-tauri/target/**", "target/**", "coverage/", "tailwind.config.ts", ".claude/"],
  },
  {
    files: ["src/**/*.{ts,tsx}"],
    ...tsBase,
    languageOptions: {
      ...tsBase.languageOptions,
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    files: ["tests/unit/**/*.test.{ts,tsx}", "tests/unit/setup.ts"],
    ...tsBase,
    languageOptions: {
      ...tsBase.languageOptions,
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        ...globals.browser,
        ...globals.node,
        ...globals.vitest,
      },
    },
  },
  {
    files: ["tests/e2e/**/*.ts", "tests/e2e/**/*.tsx"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: { ...globals.node },
      parser: parserTs,
      parserOptions: { ecmaFeatures: { jsx: false } },
    },
    plugins: { "@typescript-eslint": pluginTs },
    rules: {
      ...pluginTs.configs.recommended.rules,
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
      "no-console": ["warn", { allow: ["warn", "error"] }],
    },
  },
  {
    files: ["cli/**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: { ...globals.node },
      parser: parserTs,
      parserOptions: { ecmaFeatures: { jsx: false } },
    },
    plugins: { "@typescript-eslint": pluginTs },
    rules: {
      ...pluginTs.configs.recommended.rules,
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
      "no-console": ["warn", { allow: ["log", "warn", "error"] }],
    },
  },
];
