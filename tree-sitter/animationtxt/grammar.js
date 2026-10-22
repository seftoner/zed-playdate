/// <reference types="tree-sitter-cli/dsl" />
// Playdate launcher animation.txt: `loopCount = N`, `frames = ...` and
// `introFrames = ...`, where a frame is an image number, optionally repeated
// with `xN` (`3x4`). Any key parses; the queries name the documented ones.

module.exports = grammar({
  name: "animationtxt",
  extras: (_) => [/[ \t]+/],
  rules: {
    // One entry or comment per line; the last line may lack a newline.
    document: ($) => seq(repeat(seq(optional($._line), $._newline)), optional($._line)),
    _line: ($) => choice($.comment, $.entry),
    _newline: (_) => /\r?\n/,
    comment: (_) => token(seq("#", /[^\r\n]*/)),
    // loopCount's value is a one-frame list; the queries tell them apart.
    entry: ($) => seq(field("key", $.key), "=", field("value", $.frame_list)),
    frame_list: ($) => seq($.frame, repeat(seq(",", $.frame))),
    frame: ($) => seq(field("image", $.number), optional(field("repeat", $.repeat))),
    // One token, so `x4` is never read as a key.
    repeat: (_) => token(prec(1, /x[0-9]+/)),
    key: (_) => /[A-Za-z_][A-Za-z0-9_]*/,
    number: (_) => /[0-9]+/,
  },
});
