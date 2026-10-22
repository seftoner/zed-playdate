# Playdate Debugger for Zed

Adds a `Playdate` debug adapter for games in the Playdate Simulator. It debugs
both languages a Playdate game is written in:

- **Lua:** breakpoints, stepping, the stack, variables and evaluate. Game
  console output goes to the debug console. The Simulator is itself a Debug
  Adapter Protocol server on `127.0.0.1:55934`, and Zed connects over TCP.
- **C:** LLDB's `lldb-dap` debugs the Simulator process, which loads your
  game's native library.

A C session that launches the game and a Lua session that attaches to it can
run at the same time.

## Requirements

- The Playdate SDK, located through either of these:
  - `PLAYDATE_SDK_PATH` exported in your shell (Zed reads the login
    shell environment);
  - `sdkPath` in the debug scenario.

  A session without either fails with a message saying so.
- For C: `lldb-dap`. On macOS it comes with Xcode or the Command Line Tools
  and runs through `xcrun`. Elsewhere it must be on `PATH`.

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
  },
  {
    "label": "Debug C: Playdate Simulator",
    "adapter": "Playdate",
    "language": "c",
    "request": "launch",
    "build": "<task that builds the .pdx>",
    "gamePath": "$ZED_WORKTREE_ROOT/build/MyGame.pdx"
  },
  {
    "label": "Debug C: attach to running Simulator",
    "adapter": "Playdate",
    "language": "c",
    "request": "attach"
  }
]
```

- **`language`:** `lua` (the default) or `c`.
- **`launch`:** starts the Simulator and runs `gamePath` in it. Stopping the
  session closes the Simulator. Before a Lua launch, close any open
  Simulator: it holds the debug port.
- **`attach`:** debugs the game an open Simulator already runs. To debug C
  and Lua together, launch with `c`, then attach with `lua`.
- **`sourcePath`** (Lua only): the directory the `.pdx` was compiled from. It
  defaults to `<worktree>/Source`. When `pdc` compiles a copy (a staging
  directory), point this at the original. Otherwise the Simulator accepts
  breakpoints but never hits them.
- **`sdkPath`:** overrides `PLAYDATE_SDK_PATH`.
- **`simulatorPath`:** uses another Simulator executable.
- **`dap.Playdate.binary`** in Zed settings: uses another `lldb-dap` for C.
- **`tcp_connection`:** sets another host or port for Lua.

## Development

Install Rust through rustup. In Zed, run `zed: install dev extension` and
choose this directory.

## Licence

MIT; see `LICENSE`.
