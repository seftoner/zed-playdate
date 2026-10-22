//! Zed debug adapter for Lua in the Playdate Simulator. The Simulator is
//! itself a DAP server on 127.0.0.1:55934, and Zed connects over TCP.
//! `launch` starts the Simulator from the SDK and asks it to run `gamePath`;
//! `attach` debugs the game an open Simulator already runs. Both need
//! `sourcePath`, which defaults to <worktree>/Source.

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

struct Playdate;

/// `sdkPath` in the scenario wins over PLAYDATE_SDK_PATH from the shell.
fn sdk_path(config: &serde_json::Value, worktree: &Worktree) -> Result<String, String> {
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

fn simulator_path(sdk: &str) -> String {
    let sdk = sdk.trim_end_matches('/');
    match zed::current_platform().0 {
        Os::Mac => format!("{sdk}/bin/Playdate Simulator.app/Contents/MacOS/Playdate Simulator"),
        Os::Linux => format!("{sdk}/bin/PlaydateSimulator"),
        Os::Windows => format!("{sdk}\\bin\\PlaydateSimulator.exe"),
    }
}

fn request_kind(config: &serde_json::Value) -> Result<StartDebuggingRequestArgumentsRequest, String> {
    match config.get("request").and_then(|r| r.as_str()) {
        None | Some("launch") => Ok(StartDebuggingRequestArgumentsRequest::Launch),
        Some("attach") => Ok(StartDebuggingRequestArgumentsRequest::Attach),
        Some(other) => Err(format!("Playdate request must be \"launch\" or \"attach\", not \"{other}\"")),
    }
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
        // Launch starts the Simulator as the adapter process; Zed connects
        // once it listens and ends it with the session. Attach only connects.
        let command = if request == StartDebuggingRequestArgumentsRequest::Launch {
            let sdk = sdk_path(&configuration, worktree)?;
            let object = configuration
                .as_object_mut()
                .ok_or("Playdate debug configuration must be an object")?;
            if !object.contains_key("gamePath") {
                return Err("Playdate launch needs gamePath (the .pdx to start)".into());
            }
            object.insert("sdkPath".into(), sdk.clone().into());
            // `dap.Playdate.binary` in Zed settings overrides the Simulator.
            Some(user_provided_debug_adapter_path.unwrap_or_else(|| simulator_path(&sdk)))
        } else {
            None
        };
        let object = configuration
            .as_object_mut()
            .ok_or("Playdate debug configuration must be an object")?;
        // Without sourcePath the Simulator accepts breakpoints but never hits
        // them: pdc compiled a staged copy of Source/.
        if !object.contains_key("sourcePath") {
            object.insert(
                "sourcePath".into(),
                format!("{}/Source", worktree.root_path()).into(),
            );
        }
        let template = config.tcp_connection.unwrap_or(TcpArgumentsTemplate {
            port: None,
            host: None,
            timeout: None,
        });
        Ok(DebugAdapterBinary {
            command,
            arguments: Vec::new(),
            envs: Vec::new(),
            cwd: None,
            connection: Some(TcpArguments {
                port: template.port.unwrap_or(PORT),
                host: template.host.unwrap_or(u32::from(Ipv4Addr::LOCALHOST)),
                timeout: template.timeout.or(Some(CONNECT_TIMEOUT_MS)),
            }),
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
