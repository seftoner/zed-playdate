# Playdate Debugger for Zed

Adds a `Playdate` debug adapter: debug Lua line by line in the Playdate
Simulator, with breakpoints, stepping, the stack, variables and evaluate. Game
console output goes to the debug console.

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

## Licence

MIT; see `LICENSE`.
