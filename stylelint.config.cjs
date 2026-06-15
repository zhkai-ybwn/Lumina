module.exports = {
  root: true,
  ignoreFiles: [
    "dist/**/*",
    "src-tauri/target/**/*"
  ],
  extends: [
    "stylelint-config-standard",
    "stylelint-config-prettier"
  ],
  plugins: ["stylelint-order"],
  overrides: [
    {
      files: ["**/*.vue"],
      customSyntax: "postcss-html"
    },
    {
      files: ["**/*.{scss,sass}"],
      customSyntax: "postcss-scss",
      rules: {
        "at-rule-no-unknown": [
          true,
          {
            ignoreAtRules: [
              "each",
              "else",
              "for",
              "forward",
              "function",
              "if",
              "include",
              "mixin",
              "return",
              "use"
            ]
          }
        ],
        "no-invalid-double-slash-comments": null
      }
    }
  ],
  rules: {
    "order/properties-alphabetical-order": true,
    "no-descending-specificity": null,
    "selector-class-pattern": null
  }
};
