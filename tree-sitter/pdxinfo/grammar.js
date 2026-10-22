/// <reference types="tree-sitter-cli/dsl" />
// Playdate pdxinfo: one `key=value` per line, `#` comments. Any key parses,
// so keys newer than this grammar still highlight; the queries name the
// keys the SDK documents.

module.exports = grammar({
  name: "pdxinfo",
  extras: (_) => [],
  rules: {
    // One entry or comment per line; the last line may lack a newline.
    document: ($) => seq(repeat(seq(optional($._line), $._newline)), optional($._line)),
    _line: ($) => choice($._blank, $.comment, $.entry),
    _newline: (_) => /\r?\n/,
    _blank: (_) => /[ \t]+/,
    comment: (_) => token(seq("#", /[^\r\n]*/)),
    entry: ($) =>
      seq(
        field("key", $.key),
        optional(/[ \t]+/),
        "=",
        optional(/[ \t]+/),
        optional(field("value", $.value)),
      ),
    key: (_) => /[A-Za-z_][A-Za-z0-9_]*/,
    value: (_) => /[^ \t\r\n][^\r\n]*/,
  },
});
