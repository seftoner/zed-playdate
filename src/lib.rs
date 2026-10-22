//! Zed debug adapter for Playdate games in the Simulator.
//!
//! - `"language": "lua"` (default): the Simulator is itself a DAP server on
//!   127.0.0.1:55934, and Zed connects over TCP. `launch` starts the
//!   Simulator from the SDK and asks it to run `gamePath`; `attach` debugs
//!   the game an open Simulator already runs. `sourcePath` defaults to
//!   <worktree>/Source.
//! - `"language": "c"`: LLDB's lldb-dap (found with xcrun on macOS, on PATH
//!   elsewhere) launches the Simulator with `gamePath` or attaches to an
//!   open one.
//!
//! A C launch and a Lua attach can run at the same time.

use std::net::Ipv4Addr;

use zed_extension_api::{
    self as zed, serde_json, DebugAdapterBinary, DebugTaskDefinition, Os,
    StartDebuggingRequestArguments, StartDebuggingRequestArgumentsRequest, TcpArguments,
    TcpArgumentsTemplate, Worktree,
};

const ADAPTER: &str = "Playdate";
const PORT: u16 = 55934;
/// A cold Simulator takes a few seconds to open its debug port.
const CONNECT_TIMEOUT_MS: u64 = 15_000;

type Object = serde_json::Map<String, serde_json::Value>;

/// How Zed reaches the adapter: a process to start and/or a TCP port.
struct Transport {
    command: Option<String>,
    arguments: Vec<String>,
    connection: Option<TcpArguments>,
}

struct Playdate;

#[derive(PartialEq)]
enum Language {
    Lua,
    C,
}

fn language(config: &serde_json::Value) -> Result<Language, String> {
    match config.get("language").and_then(|l| l.as_str()) {
        None | Some("lua") => Ok(Language::Lua),
        Some("c") => Ok(Language::C),
        Some(other) => Err(format!("Playdate language must be \"lua\" or \"c\", not \"{other}\"")),
    }
}

fn request_kind(config: &serde_json::Value) -> Result<StartDebuggingRequestArgumentsRequest, String> {
    match config.get("request").and_then(|r| r.as_str()) {
        None | Some("launch") => Ok(StartDebuggingRequestArgumentsRequest::Launch),
        Some("attach") => Ok(StartDebuggingRequestArgumentsRequest::Attach),
        Some(other) => Err(format!("Playdate request must be \"launch\" or \"attach\", not \"{other}\"")),
    }
}

/// `sdkPath` in the scenario wins over PLAYDATE_SDK_PATH from the shell.
fn sdk_path(config: &Object, worktree: &Worktree) -> Result<String, String> {
    if let Some(path) = config.get("sdkPath").and_then(|p| p.as_str()) {
        return Ok(path.to_string());
    }
    worktree
        .shell_env()
        .into_iter()
        .find(|(key, value)| key == "PLAYDATE_SDK_PATH" && !value.is_empty())
        .map(|(_, value)| value)
        .ok_or_else(|| {
            "Playdate SDK not found: export PLAYDATE_SDK_PATH in your shell, \
             or set sdkPath in the debug scenario"
                .to_string()
        })
}

/// `simulatorPath` in the scenario wins over the SDK's Simulator.
fn simulator_path(config: &Object, worktree: &Worktree) -> Result<String, String> {
    if let Some(path) = config.get("simulatorPath").and_then(|p| p.as_str()) {
        return Ok(path.to_string());
    }
    let sdk = sdk_path(config, worktree)?;
    let sdk = sdk.trim_end_matches('/');
    Ok(match zed::current_platform().0 {
        Os::Mac => format!("{sdk}/bin/Playdate Simulator.app/Contents/MacOS/Playdate Simulator"),
        Os::Linux => format!("{sdk}/bin/PlaydateSimulator"),
        Os::Windows => format!("{sdk}\\bin\\PlaydateSimulator.exe"),
    })
}

fn game_path(config: &Object) -> Result<String, String> {
    config
        .get("gamePath")
        .and_then(|p| p.as_str())
        .map(str::to_string)
        .ok_or_else(|| "Playdate launch needs gamePath (the .pdx to start)".to_string())
}

/// The Simulator as Lua's adapter process for launch; attach only connects.
fn lua_binary(
    object: &mut Object,
    request: &StartDebuggingRequestArgumentsRequest,
    tcp: Option<TcpArgumentsTemplate>,
    worktree: &Worktree,
) -> Result<Transport, String> {
    let command = if *request == StartDebuggingRequestArgumentsRequest::Launch {
        game_path(object)?;
        let simulator = simulator_path(object, worktree)?;
        if let Ok(sdk) = sdk_path(object, worktree) {
            object.insert("sdkPath".into(), sdk.into());
        }
        Some(simulator)
    } else {
        None
    };
    // Without sourcePath the Simulator accepts breakpoints but never hits
    // them: pdc compiled a staged copy of Source/.
    if !object.contains_key("sourcePath") {
        object.insert("sourcePath".into(), format!("{}/Source", worktree.root_path()).into());
    }
    let tcp = tcp.unwrap_or(TcpArgumentsTemplate { port: None, host: None, timeout: None });
    let connection = TcpArguments {
        port: tcp.port.unwrap_or(PORT),
        host: tcp.host.unwrap_or(u32::from(Ipv4Addr::LOCALHOST)),
        timeout: tcp.timeout.or(Some(CONNECT_TIMEOUT_MS)),
    };
    Ok(Transport { command, arguments: Vec::new(), connection: Some(connection) })
}

/// lldb-dap over stdio; it launches the Simulator or attaches to it by path.
fn c_binary(
    object: &mut Object,
    request: &StartDebuggingRequestArgumentsRequest,
    user_provided_debug_adapter_path: Option<String>,
    worktree: &Worktree,
) -> Result<Transport, String> {
    let simulator = simulator_path(object, worktree)?;
    if *request == StartDebuggingRequestArgumentsRequest::Launch {
        let game = game_path(object)?;
        object.insert("args".into(), serde_json::json!([game]));
        if !object.contains_key("cwd") {
            object.insert("cwd".into(), worktree.root_path().into());
        }
    }
    object.insert("program".into(), simulator.into());
    // `dap.Playdate.binary` in Zed settings sets another lldb-dap.
    let (command, arguments) = match user_provided_debug_adapter_path {
        Some(path) => (path, Vec::new()),
        None => match zed::current_platform().0 {
            Os::Mac => ("/usr/bin/xcrun".to_string(), vec!["lldb-dap".to_string()]),
            _ => (
                worktree.which("lldb-dap").ok_or(
                    "lldb-dap not found on PATH: install LLDB, or set dap.Playdate.binary",
                )?,
                Vec::new(),
            ),
        },
    };
    Ok(Transport { command: Some(command), arguments, connection: None })
}

impl zed::Extension for Playdate {
    fn new() -> Self {
        Playdate
    }

    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        config: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        if adapter_name != ADAPTER {
            return Err(format!("unknown debug adapter {adapter_name}"));
        }
        let mut configuration: serde_json::Value = serde_json::from_str(&config.config)
            .map_err(|e| format!("invalid Playdate debug configuration: {e}"))?;
        let request = request_kind(&configuration)?;
        let language = language(&configuration)?;
        let object = configuration
            .as_object_mut()
            .ok_or("Playdate debug configuration must be an object")?;
        let transport = match language {
            Language::Lua => lua_binary(object, &request, config.tcp_connection, worktree)?,
            Language::C => c_binary(object, &request, user_provided_debug_adapter_path, worktree)?,
        };
        Ok(DebugAdapterBinary {
            command: transport.command,
            arguments: transport.arguments,
            envs: Vec::new(),
            cwd: None,
            connection: transport.connection,
            request_args: StartDebuggingRequestArguments {
                configuration: configuration.to_string(),
                request,
            },
        })
    }

    fn dap_request_kind(
        &mut self,
        _adapter_name: String,
        config: serde_json::Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        request_kind(&config)
    }
}

zed::register_extension!(Playdate);
