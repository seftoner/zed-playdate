# Playdate Tools for Zed

- **`Playdate` debug adapter.** Debug Lua line by line in the Playdate
  Simulator: breakpoints, stepping, stack, variables and evaluate. Game console
  output goes to the debug console.
- **Highlighting** for `pdxinfo` and `animation.txt` (launcher card and icon
  animations). Any key parses. Keys the SDK documents are keywords, unknown
  keys are properties, and values are typed: `buildNumber` is a number,
  `bundleID` and paths are special strings.

The Simulator is itself a Debug Adapter Protocol server on
`127.0.0.1:55934`, so Zed connects over TCP. For C code, debug the Simulator
process with CodeLLDB; both sessions can run at once.

## Requirements

- The Playdate SDK, located through either of these:
  - `PLAYDATE_SDK_PATH` exported in your shell (Zed reads the login
    shell environment);
  - `sdkPath` in the debug scenario.

  A `launch` without either fails with a message saying so.
- Rust through rustup, to install this as a dev extension (Zed builds it
  for `wasm32-wasip2`).

## Install

In Zed, run `zed: install dev extension` and choose this directory.

## Debug scenarios

Add these to `.zed/debug.json` in your game's repository:

```json
[
  {
    "label": "Debug Lua: Playdate Simulator",
    "adapter": "Playdate",
    "request": "launch",
    "build": "<task that builds the .pdx>",
    "gamePath": "$ZED_WORKTREE_ROOT/build/MyGame.pdx",
    "sourcePath": "$ZED_WORKTREE_ROOT/Source"
  },
  {
    "label": "Debug Lua: attach to running Simulator",
    "adapter": "Playdate",
    "request": "attach",
    "sourcePath": "$ZED_WORKTREE_ROOT/Source"
  }
]
```

- **`launch`** starts the Simulator from the SDK and asks it to run
  `gamePath`. Stopping the session closes the Simulator. If a Simulator is
  already open, close it first: it holds the debug port.
- **`attach`** connects to an open Simulator and debugs the game it runs.
- **`sourcePath`** is the directory the `.pdx` was compiled from. It
  defaults to `<worktree>/Source`. When `pdc` compiles a copy (a staging
  directory), point this at the original. Otherwise the Simulator accepts
  breakpoints but never hits them.
- **`sdkPath`** overrides `PLAYDATE_SDK_PATH`.
- **`dap.Playdate.binary`** in Zed settings sets another Simulator
  executable.
- **`tcp_connection`** in a scenario sets another host or port.

## Grammars

The tree-sitter grammars are in `tree-sitter/`. After changing a
`grammar.js`, regenerate its parser:

```sh
cd tree-sitter/pdxinfo && npx tree-sitter-cli@0.25.10 generate
```

Then commit and push. Point `rev` in `extension.toml` at that commit: Zed
fetches grammars from the repository, not from your checkout.

## Licence

MIT; see `LICENSE`.
