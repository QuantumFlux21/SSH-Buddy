use std::{
    collections::BTreeMap,
    env,
    ffi::{OsStr, OsString},
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use crate::domain::{
    normalize_tunnel_bind_host, validate_proxy_jump, validate_rdp_monitor_ids,
    validate_ssh_destination, validate_tunnel_input, AppResult, AppSettings, LaunchBinaryStatus,
    LaunchDiagnostics, RdpSettings, ServerProfile, TerminalAvailability, Tunnel, TunnelInput,
    RDP_CERTIFICATE_MODE_IGNORE, RDP_CERTIFICATE_MODE_PROMPT, RDP_CERTIFICATE_MODE_TOFU,
    RDP_SCALING_MODE_DYNAMIC_RESOLUTION, RDP_SCALING_MODE_NATIVE, RDP_SCALING_MODE_PERCENTAGE,
    RDP_SCALING_MODE_SMART_SIZING, SUPPORTED_RDP_SCALING_PERCENTS, SUPPORTED_TERMINAL_PREFERENCES,
    TERMINAL_PREFERENCE_AUTO,
};

const TERMINAL_ORDER: &[&str] = &[
    "konsole",
    "kitty",
    "alacritty",
    "wezterm",
    "gnome-terminal",
    "xterm",
];
const RDP_CLIENT_ORDER: &[&str] = &["xfreerdp3", "xfreerdp"];
const IMMEDIATE_EXIT_CHECKS: usize = 8;
const IMMEDIATE_EXIT_INTERVAL: Duration = Duration::from_millis(75);
const STDERR_LIMIT_BYTES: usize = 2_048;
const APPIMAGE_PATH_VARIABLES: &[&str] = &[
    "PATH",
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "XDG_DATA_DIRS",
    "XDG_CONFIG_DIRS",
    "GTK_PATH",
    "GTK_DATA_PREFIX",
    "GDK_PIXBUF_MODULE_FILE",
    "GIO_EXTRA_MODULES",
    "GSETTINGS_SCHEMA_DIR",
    "GI_TYPELIB_PATH",
    "GST_PLUGIN_PATH",
    "GST_PLUGIN_SYSTEM_PATH",
    "QT_PLUGIN_PATH",
    "QML2_IMPORT_PATH",
];
const APPIMAGE_RUNTIME_VARIABLES: &[&str] = &["APPIMAGE", "APPDIR", "ARGV0", "OWD"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
struct HostLaunchEnvironment {
    variables: BTreeMap<OsString, OsString>,
    sanitized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProcessObservation {
    Running,
    Exited { code: Option<i32>, stderr: String },
    SpawnFailed(String),
}

trait ProcessRunner {
    fn launch(
        &self,
        command: &ProcessCommand,
        environment: &HostLaunchEnvironment,
    ) -> ProcessObservation;
}

struct SystemProcessRunner;

pub fn build_ssh_argv(
    server: &ServerProfile,
    identity_file: Option<&str>,
) -> AppResult<Vec<String>> {
    validate_server_profile_for_launch(server)?;

    let mut argv = vec!["ssh".to_string()];
    append_profile_ssh_options(&mut argv, server, identity_file)?;

    argv.push(destination_for(server)?);

    Ok(argv)
}

pub fn build_sftp_argv(
    server: &ServerProfile,
    identity_file: Option<&str>,
) -> AppResult<Vec<String>> {
    validate_server_profile_for_launch(server)?;

    let mut argv = vec!["sftp".to_string()];
    append_profile_sftp_options(&mut argv, server, identity_file)?;
    argv.push(destination_for(server)?);

    Ok(argv)
}

pub fn public_key_path_for_identity_file(identity_file: &str) -> AppResult<String> {
    let Some(identity_file) = normalize_optional(identity_file) else {
        return Err("SSH key reference path is required".to_string());
    };

    Ok(format!("{}.pub", expand_home_path(&identity_file)))
}

pub fn build_install_public_key_argv(
    server: &ServerProfile,
    identity_file: &str,
) -> AppResult<Vec<String>> {
    validate_install_public_key_profile(server)?;
    let public_key_path = public_key_path_for_identity_file(identity_file)?;

    let mut argv = vec![
        "ssh-copy-id".to_string(),
        "-i".to_string(),
        public_key_path,
        "-p".to_string(),
        server.port.to_string(),
    ];

    if let Some(proxy_jump) = server.proxy_jump.as_deref().and_then(normalize_optional) {
        validate_proxy_jump(&proxy_jump)?;
        argv.push("-o".to_string());
        argv.push(format!("ProxyJump={proxy_jump}"));
    }

    argv.push(destination_for(server)?);

    Ok(argv)
}

pub fn build_rdp_argv(
    client: &str,
    server: &ServerProfile,
    settings: &RdpSettings,
) -> AppResult<Vec<String>> {
    validate_rdp_client(client)?;
    validate_server_profile_for_launch(server)?;
    validate_rdp_settings_for_launch(settings)?;

    let host = server.host.trim();
    validate_rdp_host(host)?;

    let mut argv = vec![client.to_string(), format!("/v:{host}:{}", settings.port)];
    append_rdp_certificate_option(&mut argv, settings)?;
    append_rdp_scaling_option(&mut argv, settings)?;

    if let Some(username) = settings.username.as_deref().and_then(normalize_optional) {
        argv.push(format!("/u:{username}"));
    }

    if let Some(domain) = settings.domain.as_deref().and_then(normalize_optional) {
        argv.push(format!("/d:{domain}"));
    }

    if settings.fullscreen {
        argv.push("/f".to_string());
    }

    if settings.multi_monitor {
        argv.push("/multimon".to_string());
    }

    if let Some(monitor_ids) = settings.monitor_ids.as_deref().and_then(normalize_optional) {
        argv.push(format!("/monitors:{monitor_ids}"));
    }

    if let Some(width) = settings.width {
        argv.push(format!("/w:{width}"));
    }

    if let Some(height) = settings.height {
        argv.push(format!("/h:{height}"));
    }

    if let Some(color_depth) = settings.color_depth {
        argv.push(format!("/bpp:{color_depth}"));
    }

    Ok(argv)
}

fn append_rdp_certificate_option(argv: &mut Vec<String>, settings: &RdpSettings) -> AppResult<()> {
    match settings.certificate_mode.as_str() {
        RDP_CERTIFICATE_MODE_PROMPT => Ok(()),
        RDP_CERTIFICATE_MODE_TOFU => {
            argv.push("/cert:tofu".to_string());
            Ok(())
        }
        RDP_CERTIFICATE_MODE_IGNORE => {
            argv.push("/cert:ignore".to_string());
            Ok(())
        }
        _ => Err("RDP certificate mode must be prompt, tofu, or ignore".to_string()),
    }
}

fn append_rdp_scaling_option(argv: &mut Vec<String>, settings: &RdpSettings) -> AppResult<()> {
    match settings.scaling_mode.as_str() {
        RDP_SCALING_MODE_NATIVE => Ok(()),
        RDP_SCALING_MODE_PERCENTAGE => {
            let Some(percent) = settings.scaling_percent else {
                return Err("RDP scaling percent is required for percentage scaling".to_string());
            };

            if !SUPPORTED_RDP_SCALING_PERCENTS.contains(&percent) {
                return Err("RDP scaling percent must be 100, 140, or 180".to_string());
            }

            argv.push(format!("/scale:{percent}"));
            Ok(())
        }
        RDP_SCALING_MODE_SMART_SIZING => {
            argv.push("/smart-sizing".to_string());
            Ok(())
        }
        RDP_SCALING_MODE_DYNAMIC_RESOLUTION => {
            argv.push("+dynamic-resolution".to_string());
            Ok(())
        }
        _ => Err(
            "RDP scaling mode must be native, percentage, smart-sizing, or dynamic-resolution"
                .to_string(),
        ),
    }
}

pub fn build_tunnel_argv(
    server: &ServerProfile,
    identity_file: Option<&str>,
    tunnel: &Tunnel,
) -> AppResult<Vec<String>> {
    validate_server_profile_for_launch(server)?;

    let tunnel_spec = local_tunnel_spec(tunnel)?;
    let mut argv = vec![
        "ssh".to_string(),
        "-N".to_string(),
        "-L".to_string(),
        tunnel_spec,
    ];
    append_profile_ssh_options(&mut argv, server, identity_file)?;
    argv.push(destination_for(server)?);

    Ok(argv)
}

fn validate_server_profile_for_launch(server: &ServerProfile) -> AppResult<()> {
    if server.host.trim().is_empty() {
        return Err("Server profile is missing a host".to_string());
    }

    if server.port == 0 {
        return Err("Server profile port must be between 1 and 65535".to_string());
    }

    Ok(())
}

fn validate_install_public_key_profile(server: &ServerProfile) -> AppResult<()> {
    validate_server_profile_for_launch(server)?;

    if server.username.trim().is_empty() {
        return Err("Username is required to install a public key".to_string());
    }

    Ok(())
}

fn validate_rdp_settings_for_launch(settings: &RdpSettings) -> AppResult<()> {
    if !settings.enabled {
        return Err("RDP is not enabled for this server".to_string());
    }

    if settings.port == 0 {
        return Err("RDP port must be between 1 and 65535".to_string());
    }

    match (settings.width, settings.height) {
        (None, None) => {}
        (Some(width), Some(height)) => {
            validate_rdp_dimension_for_launch(width, "RDP width")?;
            validate_rdp_dimension_for_launch(height, "RDP height")?;
        }
        _ => return Err("RDP width and height must be set together".to_string()),
    }

    if let Some(color_depth) = settings.color_depth {
        if !matches!(color_depth, 16 | 24 | 32) {
            return Err("RDP color depth must be 16, 24, or 32".to_string());
        }
    }

    if let Some(monitor_ids) = settings.monitor_ids.as_deref() {
        validate_rdp_monitor_ids(monitor_ids)?;
        if normalize_optional(monitor_ids).is_some() && !settings.multi_monitor {
            return Err("RDP monitor IDs require multi-monitor".to_string());
        }
    }

    validate_rdp_scaling_for_launch(settings)?;

    Ok(())
}

fn validate_rdp_scaling_for_launch(settings: &RdpSettings) -> AppResult<()> {
    match settings.scaling_mode.as_str() {
        RDP_SCALING_MODE_NATIVE
        | RDP_SCALING_MODE_SMART_SIZING
        | RDP_SCALING_MODE_DYNAMIC_RESOLUTION => {
            if settings.scaling_percent.is_some() {
                Err("RDP scaling percent is only valid for percentage scaling".to_string())
            } else {
                Ok(())
            }
        }
        RDP_SCALING_MODE_PERCENTAGE => {
            let Some(percent) = settings.scaling_percent else {
                return Err("RDP scaling percent is required for percentage scaling".to_string());
            };

            if SUPPORTED_RDP_SCALING_PERCENTS.contains(&percent) {
                Ok(())
            } else {
                Err("RDP scaling percent must be 100, 140, or 180".to_string())
            }
        }
        _ => Err(
            "RDP scaling mode must be native, percentage, smart-sizing, or dynamic-resolution"
                .to_string(),
        ),
    }
}

fn validate_rdp_dimension_for_launch(value: u16, label: &str) -> AppResult<()> {
    if (320..=16_384).contains(&value) {
        Ok(())
    } else {
        Err(format!("{label} must be between 320 and 16384"))
    }
}

fn validate_rdp_host(host: &str) -> AppResult<()> {
    if host.chars().any(char::is_whitespace) {
        return Err("RDP host must not contain whitespace".to_string());
    }

    if host.starts_with('-') {
        return Err("RDP host must not start with '-'".to_string());
    }

    if host
        .chars()
        .any(|character| !is_rdp_host_character_allowed(character))
    {
        return Err(
            "RDP host contains unsupported characters. Use a hostname or IP address.".to_string(),
        );
    }

    Ok(())
}

fn is_rdp_host_character_allowed(character: char) -> bool {
    character.is_ascii_alphanumeric() || "._-:[]".contains(character)
}

fn validate_rdp_client(client: &str) -> AppResult<()> {
    if RDP_CLIENT_ORDER.contains(&client) {
        Ok(())
    } else {
        Err("Unsupported RDP client".to_string())
    }
}

fn append_profile_ssh_options(
    argv: &mut Vec<String>,
    server: &ServerProfile,
    identity_file: Option<&str>,
) -> AppResult<()> {
    if server.port != 22 {
        argv.push("-p".to_string());
        argv.push(server.port.to_string());
    }

    if let Some(identity_file) = identity_file.and_then(normalize_optional) {
        argv.push("-i".to_string());
        argv.push(expand_home_path(&identity_file));
    }

    if let Some(proxy_jump) = server.proxy_jump.as_deref().and_then(normalize_optional) {
        validate_proxy_jump(&proxy_jump)?;
        argv.push("-J".to_string());
        argv.push(proxy_jump);
    }

    Ok(())
}

fn append_profile_sftp_options(
    argv: &mut Vec<String>,
    server: &ServerProfile,
    identity_file: Option<&str>,
) -> AppResult<()> {
    if server.port != 22 {
        argv.push("-P".to_string());
        argv.push(server.port.to_string());
    }

    if let Some(identity_file) = identity_file.and_then(normalize_optional) {
        argv.push("-i".to_string());
        argv.push(expand_home_path(&identity_file));
    }

    if let Some(proxy_jump) = server.proxy_jump.as_deref().and_then(normalize_optional) {
        validate_proxy_jump(&proxy_jump)?;
        argv.push("-o".to_string());
        argv.push(format!("ProxyJump={proxy_jump}"));
    }

    Ok(())
}

fn destination_for(server: &ServerProfile) -> AppResult<String> {
    validate_ssh_destination(&server.host, &server.username)?;

    if server.username.is_empty() {
        Ok(server.host.clone())
    } else {
        Ok(format!("{}@{}", server.username, server.host))
    }
}

fn local_tunnel_spec(tunnel: &Tunnel) -> AppResult<String> {
    validate_tunnel_input(&TunnelInput {
        label: tunnel.label.clone(),
        tunnel_type: tunnel.tunnel_type.clone(),
        local_bind_host: tunnel.local_bind_host.clone(),
        local_port: tunnel.local_port.map(u32::from),
        remote_host: tunnel.remote_host.clone(),
        remote_port: tunnel.remote_port.map(u32::from),
    })?;

    let local_bind_host = normalize_tunnel_bind_host(tunnel.local_bind_host.clone());
    let local_port = tunnel
        .local_port
        .ok_or_else(|| "Local port must be between 1 and 65535".to_string())?;
    let remote_host = tunnel
        .remote_host
        .as_deref()
        .ok_or_else(|| "Remote host is required".to_string())?
        .trim();
    let remote_port = tunnel
        .remote_port
        .ok_or_else(|| "Remote port must be between 1 and 65535".to_string())?;

    Ok(format!(
        "{local_bind_host}:{local_port}:{remote_host}:{remote_port}"
    ))
}

pub fn format_argv_for_display(argv: &[String]) -> String {
    argv.iter()
        .map(|part| shell_quote(part))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn terminal_command_for(terminal: &str, command_argv: &[String]) -> AppResult<ProcessCommand> {
    if command_argv.is_empty() {
        return Err("Launch command is empty".to_string());
    }

    let mut args = match terminal {
        // KDE documents --separate as the way to avoid process reuse and requires -e last.
        "konsole" => vec![
            "--separate".to_string(),
            "--noclose".to_string(),
            "-e".to_string(),
        ],
        "kitty" => Vec::new(),
        "alacritty" => vec!["-e".to_string()],
        "wezterm" => vec!["start".to_string(), "--".to_string()],
        "gnome-terminal" => vec!["--".to_string()],
        "xterm" => vec!["-e".to_string()],
        _ => return Err("Unsupported terminal preference".to_string()),
    };
    args.extend(command_argv.iter().cloned());

    Ok(ProcessCommand {
        program: terminal.to_string(),
        args,
    })
}

pub fn select_terminal<F>(preference: &str, available: F) -> AppResult<String>
where
    F: Fn(&str) -> bool,
{
    if !SUPPORTED_TERMINAL_PREFERENCES.contains(&preference) {
        return Err("Unsupported terminal preference".to_string());
    }

    if preference != TERMINAL_PREFERENCE_AUTO {
        if available(preference) {
            return Ok(preference.to_string());
        }

        return Err(format!(
            "Preferred terminal '{}' was not found in PATH",
            terminal_label(preference)
        ));
    }

    TERMINAL_ORDER
        .iter()
        .find(|terminal| available(terminal))
        .map(|terminal| (*terminal).to_string())
        .ok_or_else(|| {
            "No supported terminal found. Install Konsole, kitty, Alacritty, WezTerm, GNOME Terminal, or xterm.".to_string()
        })
}

pub fn launch_ssh_in_terminal(
    server: &ServerProfile,
    identity_file: Option<&str>,
    settings: &AppSettings,
) -> AppResult<LaunchDiagnostics> {
    let (diagnostics, command) =
        build_ssh_launch_diagnostics(server, identity_file, settings, command_in_path);

    Ok(spawn_with_diagnostics(diagnostics, command))
}

pub fn launch_sftp_in_terminal(
    server: &ServerProfile,
    identity_file: Option<&str>,
    settings: &AppSettings,
) -> AppResult<LaunchDiagnostics> {
    let (diagnostics, command) =
        build_sftp_launch_diagnostics(server, identity_file, settings, command_in_path);

    Ok(spawn_with_diagnostics(diagnostics, command))
}

pub fn launch_install_public_key_in_terminal(
    server: &ServerProfile,
    identity_file: &str,
    settings: &AppSettings,
) -> AppResult<LaunchDiagnostics> {
    let (diagnostics, command) = build_install_public_key_launch_diagnostics(
        server,
        identity_file,
        settings,
        command_in_path,
    );

    Ok(spawn_with_diagnostics(diagnostics, command))
}

pub fn launch_rdp(
    server: &ServerProfile,
    rdp_settings: &RdpSettings,
    app_settings: &AppSettings,
) -> AppResult<LaunchDiagnostics> {
    let (diagnostics, command) =
        build_rdp_launch_diagnostics(server, rdp_settings, app_settings, command_in_path);

    Ok(spawn_with_diagnostics(diagnostics, command))
}

pub fn launch_tunnel_in_terminal(
    server: &ServerProfile,
    identity_file: Option<&str>,
    tunnel: &Tunnel,
    settings: &AppSettings,
) -> AppResult<LaunchDiagnostics> {
    let (diagnostics, command) =
        build_tunnel_launch_diagnostics(server, identity_file, tunnel, settings, command_in_path);

    Ok(spawn_with_diagnostics(diagnostics, command))
}

pub fn test_terminal(settings: &AppSettings) -> AppResult<LaunchDiagnostics> {
    let (diagnostics, command) = build_terminal_test_diagnostics(settings, command_in_path);

    Ok(spawn_with_diagnostics(diagnostics, command))
}

pub fn build_ssh_launch_diagnostics<F>(
    server: &ServerProfile,
    identity_file: Option<&str>,
    settings: &AppSettings,
    available: F,
) -> (LaunchDiagnostics, Option<ProcessCommand>)
where
    F: Fn(&str) -> bool,
{
    let command_preview = build_ssh_argv(server, identity_file)
        .map(|argv| format_argv_for_display(&argv))
        .unwrap_or_default();
    let command = build_launch_command(server, identity_file, settings, &available);

    launch_diagnostics_for_terminal_command(
        "ssh",
        "ssh",
        "SSH",
        command_preview,
        identity_file,
        settings,
        command,
        &available,
    )
}

pub fn build_sftp_launch_diagnostics<F>(
    server: &ServerProfile,
    identity_file: Option<&str>,
    settings: &AppSettings,
    available: F,
) -> (LaunchDiagnostics, Option<ProcessCommand>)
where
    F: Fn(&str) -> bool,
{
    let command_preview = build_sftp_argv(server, identity_file)
        .map(|argv| format_argv_for_display(&argv))
        .unwrap_or_default();
    let command = build_sftp_launch_command(server, identity_file, settings, &available);

    launch_diagnostics_for_terminal_command(
        "sftp",
        "sftp",
        "SFTP",
        command_preview,
        identity_file,
        settings,
        command,
        &available,
    )
}

pub fn build_install_public_key_launch_diagnostics<F>(
    server: &ServerProfile,
    identity_file: &str,
    settings: &AppSettings,
    available: F,
) -> (LaunchDiagnostics, Option<ProcessCommand>)
where
    F: Fn(&str) -> bool,
{
    let command_preview = build_install_public_key_argv(server, identity_file)
        .map(|argv| format_argv_for_display(&argv))
        .unwrap_or_default();
    let command =
        build_install_public_key_launch_command(server, identity_file, settings, &available);
    let key_details = key_diagnostics(Some(identity_file));
    let required_binaries = terminal_binary_statuses("ssh-copy-id", &available);

    match command {
        Ok(process_command) => {
            let terminal = process_command.program.clone();
            (
                LaunchDiagnostics {
                    action_type: "install-public-key".to_string(),
                    selected_terminal_or_client: Some(terminal.clone()),
                    executable: Some(executable_display(&terminal)),
                    command_preview,
                    argv_preview: Some(process_command_to_display(&process_command)),
                    key_path: key_details.key_path,
                    key_file_exists: key_details.key_file_exists,
                    public_key_path: key_details.public_key_path,
                    public_key_file_exists: key_details.public_key_file_exists,
                    required_binaries,
                    backend_result: "preflightPassed".to_string(),
                    message: format!(
                        "SSH-Buddy is ready to start {terminal} for ssh-copy-id. Password and host-key prompts will appear in that terminal."
                    ),
                    exit_code: None,
                    stderr: None,
                    environment_sanitized: false,
                    free_rdp_executable: None,
                    launched_via_terminal: None,
                    certificate_mode: None,
                    rdp_username: None,
                    rdp_domain: None,
                    rdp_port: None,
                    rdp_fullscreen: None,
                    rdp_width: None,
                    rdp_height: None,
                    rdp_multi_monitor: None,
                    rdp_monitor_ids: None,
                    rdp_scaling_mode: None,
                    rdp_scaling_percent: None,
                    rdp_smart_sizing: None,
                    rdp_dynamic_resolution: None,
                    target_username: normalize_optional(server.username.as_str()),
                    target_host: normalize_optional(server.host.as_str()),
                    target_port: Some(server.port),
                    proxy_jump: server.proxy_jump.clone(),
                },
                Some(process_command),
            )
        }
        Err(error) => (
            LaunchDiagnostics {
                action_type: "install-public-key".to_string(),
                selected_terminal_or_client: explicit_terminal_preference(settings),
                executable: None,
                command_preview,
                argv_preview: None,
                key_path: key_details.key_path,
                key_file_exists: key_details.key_file_exists,
                public_key_path: key_details.public_key_path,
                public_key_file_exists: key_details.public_key_file_exists,
                required_binaries,
                backend_result: preflight_failure_result(&error).to_string(),
                message: error,
                exit_code: None,
                stderr: None,
                environment_sanitized: false,
                free_rdp_executable: None,
                launched_via_terminal: None,
                certificate_mode: None,
                rdp_username: None,
                rdp_domain: None,
                rdp_port: None,
                rdp_fullscreen: None,
                rdp_width: None,
                rdp_height: None,
                rdp_multi_monitor: None,
                rdp_monitor_ids: None,
                rdp_scaling_mode: None,
                rdp_scaling_percent: None,
                rdp_smart_sizing: None,
                rdp_dynamic_resolution: None,
                target_username: normalize_optional(server.username.as_str()),
                target_host: normalize_optional(server.host.as_str()),
                target_port: Some(server.port),
                proxy_jump: server.proxy_jump.clone(),
            },
            None,
        ),
    }
}

pub fn build_tunnel_launch_diagnostics<F>(
    server: &ServerProfile,
    identity_file: Option<&str>,
    tunnel: &Tunnel,
    settings: &AppSettings,
    available: F,
) -> (LaunchDiagnostics, Option<ProcessCommand>)
where
    F: Fn(&str) -> bool,
{
    let command_preview = build_tunnel_argv(server, identity_file, tunnel)
        .map(|argv| format_argv_for_display(&argv))
        .unwrap_or_default();
    let command = build_tunnel_launch_command(server, identity_file, tunnel, settings, &available);

    launch_diagnostics_for_terminal_command(
        "tunnel",
        "ssh",
        "SSH tunnel",
        command_preview,
        identity_file,
        settings,
        command,
        &available,
    )
}

pub fn build_rdp_launch_diagnostics<F>(
    server: &ServerProfile,
    rdp_settings: &RdpSettings,
    app_settings: &AppSettings,
    available: F,
) -> (LaunchDiagnostics, Option<ProcessCommand>)
where
    F: Fn(&str) -> bool,
{
    let rdp_command = build_rdp_launch_command(server, rdp_settings, &available);
    let command_preview = rdp_command
        .as_ref()
        .map(|process_command| process_command_to_display(process_command))
        .unwrap_or_default();
    let free_rdp_executable = rdp_command
        .as_ref()
        .ok()
        .map(|process_command| process_command.program.clone());
    let terminal_command = rdp_command.and_then(|rdp_command| {
        let client = rdp_command.program.clone();
        let rdp_argv = process_command_to_argv(&rdp_command);
        let terminal = select_terminal(&app_settings.terminal_preference, &available)?;
        let command = terminal_command_for(&terminal, &rdp_argv)?;
        Ok((command, terminal, client))
    });

    match terminal_command {
        Ok((process_command, terminal, client)) => {
            (
                LaunchDiagnostics {
                    action_type: "rdp".to_string(),
                    selected_terminal_or_client: Some(format!("{terminal} -> {client}")),
                    executable: Some(executable_display(&terminal)),
                    command_preview,
                    argv_preview: Some(process_command_to_display(&process_command)),
                    key_path: None,
                    key_file_exists: None,
                    public_key_path: None,
                    public_key_file_exists: None,
                    required_binaries: rdp_terminal_binary_statuses(&available),
                    backend_result: "preflightPassed".to_string(),
                    message: format!(
                        "SSH-Buddy is ready to start {terminal} for {client}. FreeRDP certificate and credential prompts should appear in that terminal."
                    ),
                    exit_code: None,
                    stderr: None,
                    environment_sanitized: false,
                    free_rdp_executable: Some(client),
                    launched_via_terminal: Some(true),
                    certificate_mode: Some(rdp_settings.certificate_mode.clone()),
                    rdp_username: rdp_settings.username.clone(),
                    rdp_domain: rdp_settings.domain.clone(),
                    rdp_port: Some(rdp_settings.port),
                    rdp_fullscreen: Some(rdp_settings.fullscreen),
                    rdp_width: rdp_settings.width,
                    rdp_height: rdp_settings.height,
                    rdp_multi_monitor: Some(rdp_settings.multi_monitor),
                    rdp_monitor_ids: rdp_settings.monitor_ids.clone(),
                    rdp_scaling_mode: Some(rdp_settings.scaling_mode.clone()),
                    rdp_scaling_percent: rdp_settings.scaling_percent,
                    rdp_smart_sizing: Some(
                        rdp_settings.scaling_mode == RDP_SCALING_MODE_SMART_SIZING,
                    ),
                    rdp_dynamic_resolution: Some(
                        rdp_settings.scaling_mode == RDP_SCALING_MODE_DYNAMIC_RESOLUTION,
                    ),
                    target_username: None,
                    target_host: None,
                    target_port: None,
                    proxy_jump: None,
                },
                Some(process_command),
            )
        }
        Err(error) => (
            LaunchDiagnostics {
                action_type: "rdp".to_string(),
                selected_terminal_or_client: None,
                executable: None,
                command_preview,
                argv_preview: None,
                key_path: None,
                key_file_exists: None,
                public_key_path: None,
                public_key_file_exists: None,
                required_binaries: rdp_terminal_binary_statuses(&available),
                backend_result: preflight_failure_result(&error).to_string(),
                message: error,
                exit_code: None,
                stderr: None,
                environment_sanitized: false,
                free_rdp_executable,
                launched_via_terminal: Some(false),
                certificate_mode: Some(rdp_settings.certificate_mode.clone()),
                rdp_username: rdp_settings.username.clone(),
                rdp_domain: rdp_settings.domain.clone(),
                rdp_port: Some(rdp_settings.port),
                rdp_fullscreen: Some(rdp_settings.fullscreen),
                rdp_width: rdp_settings.width,
                rdp_height: rdp_settings.height,
                rdp_multi_monitor: Some(rdp_settings.multi_monitor),
                rdp_monitor_ids: rdp_settings.monitor_ids.clone(),
                rdp_scaling_mode: Some(rdp_settings.scaling_mode.clone()),
                rdp_scaling_percent: rdp_settings.scaling_percent,
                rdp_smart_sizing: Some(
                    rdp_settings.scaling_mode == RDP_SCALING_MODE_SMART_SIZING,
                ),
                rdp_dynamic_resolution: Some(
                    rdp_settings.scaling_mode == RDP_SCALING_MODE_DYNAMIC_RESOLUTION,
                ),
                target_username: None,
                target_host: None,
                target_port: None,
                proxy_jump: None,
            },
            None,
        ),
    }
}

pub fn build_terminal_test_diagnostics<F>(
    settings: &AppSettings,
    available: F,
) -> (LaunchDiagnostics, Option<ProcessCommand>)
where
    F: Fn(&str) -> bool,
{
    let command_preview = format_argv_for_display(&[
        "printf".to_string(),
        "SSH-Buddy terminal test\n".to_string(),
    ]);
    let command = build_terminal_test_command(settings, &available);
    let required_binaries = terminal_binary_statuses("printf", &available);

    match command {
        Ok(process_command) => {
            let terminal = process_command.program.clone();
            (
                LaunchDiagnostics {
                    action_type: "terminal-test".to_string(),
                    selected_terminal_or_client: Some(terminal.clone()),
                    executable: Some(executable_display(&terminal)),
                    command_preview,
                    argv_preview: Some(process_command_to_display(&process_command)),
                    key_path: None,
                    key_file_exists: None,
                    public_key_path: None,
                    public_key_file_exists: None,
                    required_binaries,
                    backend_result: "preflightPassed".to_string(),
                    message: format!(
                        "SSH-Buddy is ready to start {terminal} with a harmless printf command. Konsole should stay open because SSH-Buddy launches it with --noclose."
                    ),
                    exit_code: None,
                    stderr: None,
                    environment_sanitized: false,
                    free_rdp_executable: None,
                    launched_via_terminal: None,
                    certificate_mode: None,
                    rdp_username: None,
                    rdp_domain: None,
                    rdp_port: None,
                    rdp_fullscreen: None,
                    rdp_width: None,
                    rdp_height: None,
                    rdp_multi_monitor: None,
                    rdp_monitor_ids: None,
                    rdp_scaling_mode: None,
                    rdp_scaling_percent: None,
                    rdp_smart_sizing: None,
                    rdp_dynamic_resolution: None,
                    target_username: None,
                    target_host: None,
                    target_port: None,
                    proxy_jump: None,
                },
                Some(process_command),
            )
        }
        Err(error) => (
            LaunchDiagnostics {
                action_type: "terminal-test".to_string(),
                selected_terminal_or_client: explicit_terminal_preference(settings),
                executable: None,
                command_preview,
                argv_preview: None,
                key_path: None,
                key_file_exists: None,
                public_key_path: None,
                public_key_file_exists: None,
                required_binaries,
                backend_result: preflight_failure_result(&error).to_string(),
                message: error,
                exit_code: None,
                stderr: None,
                environment_sanitized: false,
                free_rdp_executable: None,
                launched_via_terminal: None,
                certificate_mode: None,
                rdp_username: None,
                rdp_domain: None,
                rdp_port: None,
                rdp_fullscreen: None,
                rdp_width: None,
                rdp_height: None,
                rdp_multi_monitor: None,
                rdp_monitor_ids: None,
                rdp_scaling_mode: None,
                rdp_scaling_percent: None,
                rdp_smart_sizing: None,
                rdp_dynamic_resolution: None,
                target_username: None,
                target_host: None,
                target_port: None,
                proxy_jump: None,
            },
            None,
        ),
    }
}

pub fn build_terminal_test_command<F>(
    settings: &AppSettings,
    available: F,
) -> AppResult<ProcessCommand>
where
    F: Fn(&str) -> bool,
{
    if !available("printf") {
        return Err(
            "System tool 'printf' was not found in PATH. Cannot run terminal test.".to_string(),
        );
    }

    let command_argv = vec![
        "printf".to_string(),
        "SSH-Buddy terminal test\n".to_string(),
    ];
    let terminal = select_terminal(&settings.terminal_preference, available)?;

    terminal_command_for(&terminal, &command_argv)
}

fn launch_diagnostics_for_terminal_command<F>(
    action_type: &str,
    required_binary: &str,
    action_label: &str,
    command_preview: String,
    identity_file: Option<&str>,
    settings: &AppSettings,
    command: AppResult<ProcessCommand>,
    available: &F,
) -> (LaunchDiagnostics, Option<ProcessCommand>)
where
    F: Fn(&str) -> bool,
{
    let key_details = key_diagnostics(identity_file);
    let required_binaries = terminal_binary_statuses(required_binary, available);

    match command {
        Ok(process_command) => {
            let terminal = process_command.program.clone();
            (
                LaunchDiagnostics {
                    action_type: action_type.to_string(),
                    selected_terminal_or_client: Some(terminal.clone()),
                    executable: Some(executable_display(&terminal)),
                    command_preview,
                    argv_preview: Some(process_command_to_display(&process_command)),
                    key_path: key_details.key_path,
                    key_file_exists: key_details.key_file_exists,
                    public_key_path: key_details.public_key_path,
                    public_key_file_exists: key_details.public_key_file_exists,
                    required_binaries,
                    backend_result: "preflightPassed".to_string(),
                    message: format!("SSH-Buddy is ready to start {terminal} for {action_label}."),
                    exit_code: None,
                    stderr: None,
                    environment_sanitized: false,
                    free_rdp_executable: None,
                    launched_via_terminal: None,
                    certificate_mode: None,
                    rdp_username: None,
                    rdp_domain: None,
                    rdp_port: None,
                    rdp_fullscreen: None,
                    rdp_width: None,
                    rdp_height: None,
                    rdp_multi_monitor: None,
                    rdp_monitor_ids: None,
                    rdp_scaling_mode: None,
                    rdp_scaling_percent: None,
                    rdp_smart_sizing: None,
                    rdp_dynamic_resolution: None,
                    target_username: None,
                    target_host: None,
                    target_port: None,
                    proxy_jump: None,
                },
                Some(process_command),
            )
        }
        Err(error) => (
            LaunchDiagnostics {
                action_type: action_type.to_string(),
                selected_terminal_or_client: explicit_terminal_preference(settings),
                executable: None,
                command_preview,
                argv_preview: None,
                key_path: key_details.key_path,
                key_file_exists: key_details.key_file_exists,
                public_key_path: key_details.public_key_path,
                public_key_file_exists: key_details.public_key_file_exists,
                required_binaries,
                backend_result: preflight_failure_result(&error).to_string(),
                message: error,
                exit_code: None,
                stderr: None,
                environment_sanitized: false,
                free_rdp_executable: None,
                launched_via_terminal: None,
                certificate_mode: None,
                rdp_username: None,
                rdp_domain: None,
                rdp_port: None,
                rdp_fullscreen: None,
                rdp_width: None,
                rdp_height: None,
                rdp_multi_monitor: None,
                rdp_monitor_ids: None,
                rdp_scaling_mode: None,
                rdp_scaling_percent: None,
                rdp_smart_sizing: None,
                rdp_dynamic_resolution: None,
                target_username: None,
                target_host: None,
                target_port: None,
                proxy_jump: None,
            },
            None,
        ),
    }
}

fn spawn_with_diagnostics(
    diagnostics: LaunchDiagnostics,
    command: Option<ProcessCommand>,
) -> LaunchDiagnostics {
    let environment = host_launch_environment();
    spawn_with_diagnostics_using(diagnostics, command, &SystemProcessRunner, &environment)
}

fn spawn_with_diagnostics_using<R: ProcessRunner>(
    mut diagnostics: LaunchDiagnostics,
    command: Option<ProcessCommand>,
    runner: &R,
    environment: &HostLaunchEnvironment,
) -> LaunchDiagnostics {
    let Some(command) = command else {
        return diagnostics;
    };

    let Some(executable) = command_path_in(
        &command.program,
        environment.variables.get(OsStr::new("PATH")),
    ) else {
        diagnostics.backend_result = "notFound".to_string();
        diagnostics.message = format!(
            "The selected terminal '{}' was no longer available when launch was attempted. Recheck terminal availability and try again.",
            diagnostics
                .selected_terminal_or_client
                .as_deref()
                .unwrap_or(&command.program)
        );
        diagnostics.executable = None;
        return diagnostics;
    };

    let resolved_command = ProcessCommand {
        program: executable.to_string_lossy().into_owned(),
        args: command.args,
    };
    diagnostics.executable = Some(resolved_command.program.clone());
    diagnostics.argv_preview = Some(process_command_to_display(&resolved_command));
    diagnostics.environment_sanitized = environment.sanitized;

    apply_process_observation(
        &mut diagnostics,
        runner.launch(&resolved_command, environment),
    );

    diagnostics
}

fn apply_process_observation(diagnostics: &mut LaunchDiagnostics, observation: ProcessObservation) {
    let subject = diagnostics
        .selected_terminal_or_client
        .as_deref()
        .unwrap_or("external terminal");

    match observation {
        ProcessObservation::Running => {
            diagnostics.backend_result = "started".to_string();
            diagnostics.message = format!(
                "Started {subject}; it remained running through the immediate-failure check. This confirms the terminal launch, not an SSH or RDP connection."
            );
        }
        ProcessObservation::Exited {
            code: Some(0),
            stderr,
        } => {
            diagnostics.backend_result = "started".to_string();
            diagnostics.exit_code = Some(0);
            diagnostics.stderr = normalize_stderr(&stderr);
            diagnostics.message = format!(
                "Started {subject}; its launcher exited successfully during the immediate-failure check after handing off. This confirms the terminal launch request, not an SSH or RDP connection."
            );
        }
        ProcessObservation::Exited { code, stderr } => {
            diagnostics.backend_result = "exitedImmediately".to_string();
            diagnostics.exit_code = code;
            diagnostics.stderr = normalize_stderr(&stderr);
            let status = code
                .map(|value| format!("exit code {value}"))
                .unwrap_or_else(|| "a terminating signal".to_string());
            let stderr_hint = diagnostics
                .stderr
                .as_deref()
                .map(|value| format!(" Error: {value}"))
                .unwrap_or_default();
            diagnostics.message = format!(
                "{subject} exited immediately with {status}.{stderr_hint} Check the executable and AppImage troubleshooting guidance."
            );
        }
        ProcessObservation::SpawnFailed(error) => {
            diagnostics.backend_result = "spawnFailed".to_string();
            diagnostics.message = format!("Failed to launch {subject}: {error}");
        }
    }
}

impl ProcessRunner for SystemProcessRunner {
    fn launch(
        &self,
        command: &ProcessCommand,
        environment: &HostLaunchEnvironment,
    ) -> ProcessObservation {
        let mut process = Command::new(&command.program);
        process
            .args(&command.args)
            .env_clear()
            .envs(environment.variables.iter())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let mut child = match process.spawn() {
            Ok(child) => child,
            Err(error) => return ProcessObservation::SpawnFailed(error.to_string()),
        };

        let stderr_reader = child.stderr.take().map(|mut stderr| {
            thread::spawn(move || {
                let mut captured = Vec::new();
                let mut buffer = [0_u8; 512];
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(read) => {
                            let remaining = STDERR_LIMIT_BYTES.saturating_sub(captured.len());
                            captured.extend_from_slice(&buffer[..read.min(remaining)]);
                        }
                    }
                }
                String::from_utf8_lossy(&captured).into_owned()
            })
        });

        for _ in 0..IMMEDIATE_EXIT_CHECKS {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let stderr = stderr_reader
                        .and_then(|reader| reader.join().ok())
                        .unwrap_or_default();
                    return ProcessObservation::Exited {
                        code: status.code(),
                        stderr,
                    };
                }
                Ok(None) => thread::sleep(IMMEDIATE_EXIT_INTERVAL),
                Err(error) => return ProcessObservation::SpawnFailed(error.to_string()),
            }
        }

        ProcessObservation::Running
    }
}

fn normalize_stderr(stderr: &str) -> Option<String> {
    let mut normalized = String::new();
    for character in stderr
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'))
    {
        if normalized.len() + character.len_utf8() > STDERR_LIMIT_BYTES {
            break;
        }
        normalized.push(character);
    }
    let normalized = normalized.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}

fn preflight_failure_result(error: &str) -> &'static str {
    if error.contains("was not found in PATH") || error.starts_with("No supported ") {
        "notFound"
    } else {
        "preflightFailed"
    }
}

struct KeyDiagnostics {
    key_path: Option<String>,
    key_file_exists: Option<bool>,
    public_key_path: Option<String>,
    public_key_file_exists: Option<bool>,
}

fn key_diagnostics(identity_file: Option<&str>) -> KeyDiagnostics {
    let Some(identity_file) = identity_file.and_then(normalize_optional) else {
        return KeyDiagnostics {
            key_path: None,
            key_file_exists: None,
            public_key_path: None,
            public_key_file_exists: None,
        };
    };

    let expanded = expand_home_path(&identity_file);
    let public_key_path = format!("{expanded}.pub");

    KeyDiagnostics {
        key_path: Some(expanded.clone()),
        key_file_exists: Some(path_is_file(&expanded)),
        public_key_path: Some(public_key_path.clone()),
        public_key_file_exists: Some(path_is_file(&public_key_path)),
    }
}

fn terminal_binary_statuses<F>(action_binary: &str, available: &F) -> Vec<LaunchBinaryStatus>
where
    F: Fn(&str) -> bool,
{
    let mut names = vec![action_binary.to_string()];
    names.extend(
        TERMINAL_ORDER
            .iter()
            .map(|terminal| (*terminal).to_string()),
    );
    names
        .into_iter()
        .map(|name| LaunchBinaryStatus {
            exists: available(&name),
            executable: command_path(&name).map(|path| path.to_string_lossy().into_owned()),
            name,
        })
        .collect()
}

fn rdp_terminal_binary_statuses<F>(available: &F) -> Vec<LaunchBinaryStatus>
where
    F: Fn(&str) -> bool,
{
    let mut names = RDP_CLIENT_ORDER
        .iter()
        .map(|client| (*client).to_string())
        .collect::<Vec<_>>();
    names.extend(
        TERMINAL_ORDER
            .iter()
            .map(|terminal| (*terminal).to_string()),
    );
    names
        .into_iter()
        .map(|name| LaunchBinaryStatus {
            exists: available(&name),
            executable: command_path(&name).map(|path| path.to_string_lossy().into_owned()),
            name,
        })
        .collect()
}

fn explicit_terminal_preference(settings: &AppSettings) -> Option<String> {
    if settings.terminal_preference == TERMINAL_PREFERENCE_AUTO {
        None
    } else {
        Some(settings.terminal_preference.clone())
    }
}

fn process_command_to_display(command: &ProcessCommand) -> String {
    format_argv_for_display(&process_command_to_argv(command))
}

fn process_command_to_argv(command: &ProcessCommand) -> Vec<String> {
    let mut argv = vec![command.program.clone()];
    argv.extend(command.args.iter().cloned());
    argv
}

pub fn build_launch_command<F>(
    server: &ServerProfile,
    identity_file: Option<&str>,
    settings: &AppSettings,
    available: F,
) -> AppResult<ProcessCommand>
where
    F: Fn(&str) -> bool,
{
    if !available("ssh") {
        return Err(
            "OpenSSH client 'ssh' was not found in PATH. Install OpenSSH and try again."
                .to_string(),
        );
    }

    validate_identity_file_path(identity_file)?;
    let ssh_argv = build_ssh_argv(server, identity_file)?;
    let terminal = select_terminal(&settings.terminal_preference, available)?;

    terminal_command_for(&terminal, &ssh_argv)
}

pub fn build_sftp_launch_command<F>(
    server: &ServerProfile,
    identity_file: Option<&str>,
    settings: &AppSettings,
    available: F,
) -> AppResult<ProcessCommand>
where
    F: Fn(&str) -> bool,
{
    if !available("sftp") {
        return Err(
            "OpenSSH SFTP client 'sftp' was not found in PATH. Install OpenSSH and try again."
                .to_string(),
        );
    }

    validate_identity_file_path(identity_file)?;
    let sftp_argv = build_sftp_argv(server, identity_file)?;
    let terminal = select_terminal(&settings.terminal_preference, available)?;

    terminal_command_for(&terminal, &sftp_argv)
}

pub fn build_install_public_key_launch_command<F>(
    server: &ServerProfile,
    identity_file: &str,
    settings: &AppSettings,
    available: F,
) -> AppResult<ProcessCommand>
where
    F: Fn(&str) -> bool,
{
    if !available("ssh-copy-id") {
        return Err(
            "OpenSSH tool 'ssh-copy-id' was not found in PATH. Install the OpenSSH package that provides ssh-copy-id and try again."
                .to_string(),
        );
    }

    validate_public_key_file_path(identity_file)?;
    let install_argv = build_install_public_key_argv(server, identity_file)?;
    let terminal = select_terminal(&settings.terminal_preference, available)?;

    terminal_command_for(&terminal, &install_argv)
}

pub fn select_rdp_client<F>(available: F) -> AppResult<String>
where
    F: Fn(&str) -> bool,
{
    RDP_CLIENT_ORDER
        .iter()
        .find(|client| available(client))
        .map(|client| (*client).to_string())
        .ok_or_else(|| {
            "No supported RDP client found. Install FreeRDP xfreerdp3 or xfreerdp.".to_string()
        })
}

pub fn build_rdp_launch_command<F>(
    server: &ServerProfile,
    settings: &RdpSettings,
    available: F,
) -> AppResult<ProcessCommand>
where
    F: Fn(&str) -> bool,
{
    let client = select_rdp_client(available)?;
    let argv = build_rdp_argv(&client, server, settings)?;
    let mut args = argv;
    let program = args.remove(0);

    Ok(ProcessCommand { program, args })
}

pub fn build_tunnel_launch_command<F>(
    server: &ServerProfile,
    identity_file: Option<&str>,
    tunnel: &Tunnel,
    settings: &AppSettings,
    available: F,
) -> AppResult<ProcessCommand>
where
    F: Fn(&str) -> bool,
{
    if !available("ssh") {
        return Err(
            "OpenSSH client 'ssh' was not found in PATH. Install OpenSSH and try again."
                .to_string(),
        );
    }

    validate_identity_file_path(identity_file)?;
    let ssh_argv = build_tunnel_argv(server, identity_file, tunnel)?;
    let terminal = select_terminal(&settings.terminal_preference, available)?;

    terminal_command_for(&terminal, &ssh_argv)
}

pub(crate) fn command_in_path(command: &str) -> bool {
    command_path(command).is_some()
}

fn command_path(command: &str) -> Option<PathBuf> {
    let environment = host_launch_environment();
    command_path_in(command, environment.variables.get(OsStr::new("PATH")))
}

fn command_path_in(command: &str, path_var: Option<&OsString>) -> Option<PathBuf> {
    command_path_in_with(command, path_var, is_executable_file)
}

fn command_path_in_with<F>(
    command: &str,
    path_var: Option<&OsString>,
    is_executable: F,
) -> Option<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    let direct_path = Path::new(command);
    if direct_path.is_absolute() || direct_path.components().count() > 1 {
        return is_executable(direct_path).then(|| direct_path.to_path_buf());
    }

    let path_var = path_var?;

    env::split_paths(path_var).find_map(|path| {
        let candidate = path.join(command);
        if is_executable(&candidate) {
            Some(candidate)
        } else {
            None
        }
    })
}

fn executable_display(program: &str) -> String {
    command_path(program)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| program.to_string())
}

pub fn terminal_availability() -> Vec<TerminalAvailability> {
    let environment = host_launch_environment();
    terminal_availability_with(|terminal| {
        command_path_in(terminal, environment.variables.get(OsStr::new("PATH")))
    })
}

fn terminal_availability_with<F>(resolver: F) -> Vec<TerminalAvailability>
where
    F: Fn(&str) -> Option<PathBuf>,
{
    TERMINAL_ORDER
        .iter()
        .map(|terminal| {
            let executable = resolver(terminal).map(|path| path.to_string_lossy().into_owned());
            TerminalAvailability {
                preference: (*terminal).to_string(),
                label: terminal_label(terminal).to_string(),
                available: executable.is_some(),
                executable,
            }
        })
        .collect()
}

fn host_launch_environment() -> HostLaunchEnvironment {
    host_launch_environment_from(env::vars_os())
}

fn host_launch_environment_from<I>(variables: I) -> HostLaunchEnvironment
where
    I: IntoIterator<Item = (OsString, OsString)>,
{
    let mut variables = variables.into_iter().collect::<BTreeMap<_, _>>();
    let appdir = variables
        .get(OsStr::new("APPDIR"))
        .filter(|value| !value.is_empty())
        .map(|value| PathBuf::from(value.as_os_str()));
    let is_appimage = appdir.is_some() || variables.contains_key(OsStr::new("APPIMAGE"));

    if !is_appimage {
        return HostLaunchEnvironment {
            variables,
            sanitized: false,
        };
    }

    for name in APPIMAGE_PATH_VARIABLES {
        restore_or_filter_appimage_variable(&mut variables, name, appdir.as_deref());
    }

    for name in APPIMAGE_RUNTIME_VARIABLES {
        variables.remove(OsStr::new(name));
    }

    HostLaunchEnvironment {
        variables,
        sanitized: true,
    }
}

fn restore_or_filter_appimage_variable(
    variables: &mut BTreeMap<OsString, OsString>,
    name: &str,
    appdir: Option<&Path>,
) {
    let original_name = format!("{name}_ORIG");
    if let Some(original) = variables.remove(OsStr::new(original_name.as_str())) {
        if original.is_empty() {
            variables.remove(OsStr::new(name));
        } else {
            variables.insert(OsString::from(name), original);
        }
        return;
    }

    let Some(appdir) = appdir else {
        return;
    };
    let Some(current) = variables.get(OsStr::new(name)).cloned() else {
        return;
    };

    let entries = if name == "LD_PRELOAD" {
        split_ld_preload_paths(&current)
    } else {
        env::split_paths(&current).collect()
    };
    let filtered = entries
        .into_iter()
        .filter(|entry| !entry.starts_with(appdir))
        .collect::<Vec<_>>();
    match env::join_paths(filtered) {
        Ok(value) if !value.is_empty() => {
            variables.insert(OsString::from(name), value);
        }
        _ => {
            variables.remove(OsStr::new(name));
        }
    }
}

#[cfg(unix)]
fn split_ld_preload_paths(value: &OsStr) -> Vec<PathBuf> {
    use std::os::unix::ffi::OsStrExt;

    value
        .as_bytes()
        .split(|byte| *byte == b':' || byte.is_ascii_whitespace())
        .filter(|entry| !entry.is_empty())
        .map(|entry| PathBuf::from(OsStr::from_bytes(entry)))
        .collect()
}

#[cfg(not(unix))]
fn split_ld_preload_paths(value: &OsStr) -> Vec<PathBuf> {
    value
        .to_string_lossy()
        .split(|character: char| character == ':' || character.is_ascii_whitespace())
        .filter(|entry| !entry.is_empty())
        .map(PathBuf::from)
        .collect()
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };

    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

fn path_is_file(path: &str) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
}

fn expand_home_path(path: &str) -> String {
    expand_home_path_with(path, env::var_os("HOME"))
}

fn validate_identity_file_path(identity_file: Option<&str>) -> AppResult<()> {
    let Some(identity_file) = identity_file.and_then(normalize_optional) else {
        return Ok(());
    };
    let expanded = expand_home_path(&identity_file);
    let path = Path::new(&expanded);
    let metadata = fs::metadata(path)
        .map_err(|_| format!("Selected SSH key file was not found: {expanded}"))?;

    if !metadata.is_file() {
        return Err(format!("Selected SSH key path is not a file: {expanded}"));
    }

    Ok(())
}

pub fn validate_public_key_file_path(identity_file: &str) -> AppResult<()> {
    let private_key_path = expand_home_path(
        &normalize_optional(identity_file)
            .ok_or_else(|| "SSH key reference path is required".to_string())?,
    );
    let public_key_path = public_key_path_for_identity_file(identity_file)?;
    let metadata = fs::metadata(&public_key_path).map_err(|_| {
        format!(
            "Public key file not found. Create it with: ssh-keygen -y -f {} > {}",
            shell_quote(&private_key_path),
            shell_quote(&public_key_path)
        )
    })?;

    if !metadata.is_file() {
        return Err(format!("Public key path is not a file: {public_key_path}"));
    }

    Ok(())
}

fn expand_home_path_with(path: &str, home: Option<OsString>) -> String {
    if path != "~" && !path.starts_with("~/") {
        return path.to_string();
    }

    let Some(home) = home else {
        return path.to_string();
    };
    let home = PathBuf::from(home);

    if path == "~" {
        return home.to_string_lossy().into_owned();
    }

    home.join(&path[2..]).to_string_lossy().into_owned()
}

fn normalize_optional(value: &str) -> Option<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "@%_+=:,./-".contains(character))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn terminal_label(terminal: &str) -> &str {
    match terminal {
        "konsole" => "Konsole",
        "kitty" => "kitty",
        "alacritty" => "Alacritty",
        "wezterm" => "WezTerm",
        "gnome-terminal" => "GNOME Terminal",
        "xterm" => "xterm",
        _ => terminal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Tag;
    use tempfile::tempdir;

    struct FakeProcessRunner {
        observation: ProcessObservation,
    }

    impl ProcessRunner for FakeProcessRunner {
        fn launch(
            &self,
            _command: &ProcessCommand,
            _environment: &HostLaunchEnvironment,
        ) -> ProcessObservation {
            self.observation.clone()
        }
    }

    fn sample_server() -> ServerProfile {
        ServerProfile {
            id: "srv".to_string(),
            display_name: "NAS".to_string(),
            host: "nas.local".to_string(),
            port: 2222,
            username: "admin".to_string(),
            identity_file_id: None,
            proxy_jump: None,
            group_id: None,
            notes: None,
            favorite: false,
            tags: Vec::<Tag>::new(),
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        }
    }

    fn sample_settings() -> AppSettings {
        AppSettings {
            terminal_preference: TERMINAL_PREFERENCE_AUTO.to_string(),
            safety_warnings_enabled: true,
            start_minimized: false,
            close_to_tray: false,
        }
    }

    fn sample_tunnel() -> Tunnel {
        Tunnel {
            id: "tun".to_string(),
            server_profile_id: "srv".to_string(),
            label: "Postgres".to_string(),
            tunnel_type: "local".to_string(),
            local_bind_host: Some("127.0.0.1".to_string()),
            local_port: Some(15432),
            remote_host: Some("db.internal".to_string()),
            remote_port: Some(5432),
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        }
    }

    fn sample_rdp_settings() -> RdpSettings {
        RdpSettings {
            server_profile_id: "srv".to_string(),
            enabled: true,
            username: Some("rdpuser".to_string()),
            domain: Some("LAB".to_string()),
            port: 3390,
            certificate_mode: RDP_CERTIFICATE_MODE_TOFU.to_string(),
            fullscreen: false,
            multi_monitor: true,
            monitor_ids: Some("0,1".to_string()),
            width: Some(1920),
            height: Some(1080),
            color_depth: Some(32),
            scaling_mode: RDP_SCALING_MODE_NATIVE.to_string(),
            scaling_percent: None,
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        }
    }

    fn assert_all_ssh_builders_reject(server: &ServerProfile, expected: &str) {
        assert_eq!(build_ssh_argv(server, None).unwrap_err(), expected);
        assert_eq!(build_sftp_argv(server, None).unwrap_err(), expected);
        assert_eq!(
            build_install_public_key_argv(server, "/home/user/.ssh/id_ed25519").unwrap_err(),
            expected
        );
        assert_eq!(
            build_tunnel_argv(server, None, &sample_tunnel()).unwrap_err(),
            expected
        );
    }

    #[test]
    fn builds_ssh_argv_with_profile_fields() {
        let argv = build_ssh_argv(&sample_server(), Some("/home/user/.ssh/id_ed25519")).unwrap();

        assert_eq!(
            argv,
            vec![
                "ssh",
                "-p",
                "2222",
                "-i",
                "/home/user/.ssh/id_ed25519",
                "admin@nas.local"
            ]
        );
    }

    #[test]
    fn builds_ssh_argv_with_default_user_key_and_port() {
        let mut server = sample_server();
        server.port = 22;
        server.username = String::new();

        let argv = build_ssh_argv(&server, None).unwrap();

        assert_eq!(argv, vec!["ssh", "nas.local"]);
    }

    #[test]
    fn builds_all_ssh_destinations_for_scoped_ipv6() {
        let mut server = sample_server();
        server.host = "fe80::1%eth0".to_string();

        for argv in [
            build_ssh_argv(&server, None).unwrap(),
            build_sftp_argv(&server, None).unwrap(),
            build_install_public_key_argv(&server, "/home/user/.ssh/id_ed25519").unwrap(),
            build_tunnel_argv(&server, None, &sample_tunnel()).unwrap(),
        ] {
            assert_eq!(argv.last().map(String::as_str), Some("admin@fe80::1%eth0"));
        }

        server.host = "prod_web+blue".to_string();
        assert_eq!(
            build_ssh_argv(&server, None)
                .unwrap()
                .last()
                .map(String::as_str),
            Some("admin@prod_web+blue")
        );
    }

    #[test]
    fn rejects_unsafe_destinations_for_all_ssh_actions() {
        let mut server = sample_server();
        server.host = "-oProxyCommand=touch".to_string();
        assert_all_ssh_builders_reject(&server, "Host must not start with '-'");

        let mut server = sample_server();
        server.username = "-Fmalicious-config".to_string();
        assert_all_ssh_builders_reject(&server, "Username must not start with '-'");

        let mut server = sample_server();
        server.host = "nas local".to_string();
        assert_all_ssh_builders_reject(
            &server,
            "Host must not contain whitespace or control characters",
        );

        let mut server = sample_server();
        server.username = "admin\u{7}".to_string();
        assert_all_ssh_builders_reject(
            &server,
            "Username must not contain whitespace or control characters",
        );
    }

    #[test]
    fn builds_ssh_argv_with_proxy_jump() {
        let mut server = sample_server();
        server.proxy_jump = Some("user@bastion:22,jump2".to_string());

        let argv = build_ssh_argv(&server, None).unwrap();

        assert_eq!(
            argv,
            vec![
                "ssh",
                "-p",
                "2222",
                "-J",
                "user@bastion:22,jump2",
                "admin@nas.local"
            ]
        );
    }

    #[test]
    fn builds_sftp_argv_with_profile_fields() {
        let argv = build_sftp_argv(&sample_server(), Some("/home/user/.ssh/id_ed25519")).unwrap();

        assert_eq!(
            argv,
            vec![
                "sftp",
                "-P",
                "2222",
                "-i",
                "/home/user/.ssh/id_ed25519",
                "admin@nas.local"
            ]
        );
    }

    #[test]
    fn builds_sftp_argv_with_proxy_jump_option() {
        let mut server = sample_server();
        server.proxy_jump = Some("user@bastion:22,jump2".to_string());

        let argv = build_sftp_argv(&server, None).unwrap();

        assert_eq!(
            argv,
            vec![
                "sftp",
                "-P",
                "2222",
                "-o",
                "ProxyJump=user@bastion:22,jump2",
                "admin@nas.local"
            ]
        );
    }

    #[test]
    fn resolves_public_key_path_from_identity_file() {
        assert_eq!(
            public_key_path_for_identity_file("/home/user/.ssh/id_ed25519_homelab").unwrap(),
            "/home/user/.ssh/id_ed25519_homelab.pub"
        );
    }

    #[test]
    fn builds_install_public_key_argv_with_profile_fields() {
        let mut server = sample_server();
        server.proxy_jump = Some("user@bastion:22".to_string());

        let argv =
            build_install_public_key_argv(&server, "/home/user/.ssh/id_ed25519_homelab").unwrap();

        assert_eq!(
            argv,
            vec![
                "ssh-copy-id",
                "-i",
                "/home/user/.ssh/id_ed25519_homelab.pub",
                "-p",
                "2222",
                "-o",
                "ProxyJump=user@bastion:22",
                "admin@nas.local"
            ]
        );
    }

    #[test]
    fn builds_install_public_key_argv_with_default_port_and_quoted_display() {
        let mut server = sample_server();
        server.port = 22;

        let argv = build_install_public_key_argv(&server, "/home/alex/.ssh/lab key").unwrap();
        assert_eq!(
            argv,
            vec![
                "ssh-copy-id",
                "-i",
                "/home/alex/.ssh/lab key.pub",
                "-p",
                "22",
                "admin@nas.local"
            ]
        );
        assert_eq!(
            format_argv_for_display(&argv),
            "ssh-copy-id -i '/home/alex/.ssh/lab key.pub' -p 22 admin@nas.local"
        );
    }

    #[test]
    fn rejects_install_public_key_without_username() {
        let mut server = sample_server();
        server.username = " ".to_string();

        assert_eq!(
            build_install_public_key_argv(&server, "/home/user/.ssh/id_ed25519").unwrap_err(),
            "Username is required to install a public key"
        );
    }

    #[test]
    fn rejects_legacy_whitespace_username_for_other_ssh_actions() {
        let mut server = sample_server();
        server.username = " ".to_string();
        let expected = "Username must not contain whitespace or control characters";

        assert_eq!(build_ssh_argv(&server, None).unwrap_err(), expected);
        assert_eq!(build_sftp_argv(&server, None).unwrap_err(), expected);
        assert_eq!(
            build_tunnel_argv(&server, None, &sample_tunnel()).unwrap_err(),
            expected
        );
    }

    #[test]
    fn builds_rdp_argv_with_profile_fields() {
        let argv = build_rdp_argv("xfreerdp3", &sample_server(), &sample_rdp_settings()).unwrap();

        assert_eq!(
            argv,
            vec![
                "xfreerdp3",
                "/v:nas.local:3390",
                "/cert:tofu",
                "/u:rdpuser",
                "/d:LAB",
                "/multimon",
                "/monitors:0,1",
                "/w:1920",
                "/h:1080",
                "/bpp:32"
            ]
        );
        assert!(!argv.iter().any(|arg| arg.starts_with("/p:")));
    }

    #[test]
    fn builds_rdp_argv_with_fullscreen_and_without_username() {
        let mut settings = sample_rdp_settings();
        settings.username = None;
        settings.domain = None;
        settings.port = 3389;
        settings.certificate_mode = RDP_CERTIFICATE_MODE_PROMPT.to_string();
        settings.fullscreen = true;
        settings.multi_monitor = false;
        settings.monitor_ids = None;
        settings.width = None;
        settings.height = None;
        settings.color_depth = None;

        let argv = build_rdp_argv("xfreerdp", &sample_server(), &settings).unwrap();

        assert_eq!(argv, vec!["xfreerdp", "/v:nas.local:3389", "/f"]);
    }

    #[test]
    fn builds_rdp_argv_with_ignore_certificate_mode() {
        let mut settings = sample_rdp_settings();
        settings.certificate_mode = RDP_CERTIFICATE_MODE_IGNORE.to_string();

        let argv = build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap();

        assert!(argv.contains(&"/cert:ignore".to_string()));
        assert!(!argv.iter().any(|arg| arg.starts_with("/p:")));
    }

    #[test]
    fn builds_rdp_argv_with_scaling_modes() {
        let mut settings = sample_rdp_settings();
        settings.scaling_mode = RDP_SCALING_MODE_PERCENTAGE.to_string();
        settings.scaling_percent = Some(140);

        let argv = build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap();

        assert!(argv.contains(&"/scale:140".to_string()));
        assert!(!argv.iter().any(|arg| arg.starts_with("/p:")));

        settings.scaling_mode = RDP_SCALING_MODE_SMART_SIZING.to_string();
        settings.scaling_percent = None;
        let argv = build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap();
        assert!(argv.contains(&"/smart-sizing".to_string()));
        assert!(!argv.iter().any(|arg| arg.starts_with("/scale:")));

        settings.scaling_mode = RDP_SCALING_MODE_DYNAMIC_RESOLUTION.to_string();
        let argv = build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap();
        assert!(argv.contains(&"+dynamic-resolution".to_string()));
        assert!(!argv.iter().any(|arg| arg.starts_with("/p:")));
    }

    #[test]
    fn builds_tunnel_argv_with_local_forward() {
        let argv = build_tunnel_argv(&sample_server(), None, &sample_tunnel()).unwrap();

        assert_eq!(
            argv,
            vec![
                "ssh",
                "-N",
                "-L",
                "127.0.0.1:15432:db.internal:5432",
                "-p",
                "2222",
                "admin@nas.local"
            ]
        );
    }

    #[test]
    fn formats_tunnel_command_with_identity_file_and_proxy_jump() {
        let mut server = sample_server();
        server.proxy_jump = Some("user@bastion:22".to_string());

        let command = format_argv_for_display(
            &build_tunnel_argv(&server, Some("/home/alex/.ssh/lab key"), &sample_tunnel()).unwrap(),
        );

        assert_eq!(
            command,
            "ssh -N -L 127.0.0.1:15432:db.internal:5432 -p 2222 -i '/home/alex/.ssh/lab key' -J user@bastion:22 admin@nas.local"
        );
    }

    #[test]
    fn formats_sftp_command_with_identity_file_and_proxy_jump() {
        let mut server = sample_server();
        server.proxy_jump = Some("user@bastion:22".to_string());

        let command = format_argv_for_display(
            &build_sftp_argv(&server, Some("/home/alex/.ssh/lab key")).unwrap(),
        );

        assert_eq!(
            command,
            "sftp -P 2222 -i '/home/alex/.ssh/lab key' -o ProxyJump=user@bastion:22 admin@nas.local"
        );
    }

    #[test]
    fn formats_rdp_command_with_quoting() {
        let mut settings = sample_rdp_settings();
        settings.username = Some("Lab User".to_string());

        let command = format_argv_for_display(
            &build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap(),
        );

        assert_eq!(
            command,
            "xfreerdp3 /v:nas.local:3390 /cert:tofu '/u:Lab User' /d:LAB /multimon /monitors:0,1 /w:1920 /h:1080 /bpp:32"
        );
    }

    #[test]
    fn rejects_incomplete_server_profiles() {
        let mut server = sample_server();
        server.host = " ".to_string();
        assert_eq!(
            build_ssh_argv(&server, None).unwrap_err(),
            "Server profile is missing a host"
        );

        let mut server = sample_server();
        server.port = 0;
        assert_eq!(
            build_ssh_argv(&server, None).unwrap_err(),
            "Server profile port must be between 1 and 65535"
        );

        let mut server = sample_server();
        server.proxy_jump = Some("bastion;touch".to_string());
        assert_eq!(
            build_ssh_argv(&server, None).unwrap_err(),
            "ProxyJump contains unsupported characters. Use OpenSSH host specs like user@bastion:22."
        );
    }

    #[test]
    fn rejects_invalid_rdp_settings_for_launch() {
        let mut settings = sample_rdp_settings();
        settings.enabled = false;
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP is not enabled for this server"
        );

        let mut settings = sample_rdp_settings();
        settings.port = 0;
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP port must be between 1 and 65535"
        );

        let mut settings = sample_rdp_settings();
        settings.height = None;
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP width and height must be set together"
        );

        let mut settings = sample_rdp_settings();
        settings.color_depth = Some(8);
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP color depth must be 16, 24, or 32"
        );

        let mut settings = sample_rdp_settings();
        settings.monitor_ids = Some("0;1".to_string());
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP monitor IDs must be comma-separated monitor numbers"
        );

        let mut settings = sample_rdp_settings();
        settings.multi_monitor = false;
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP monitor IDs require multi-monitor"
        );

        let mut settings = sample_rdp_settings();
        settings.scaling_mode = "stretch".to_string();
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP scaling mode must be native, percentage, smart-sizing, or dynamic-resolution"
        );

        let mut settings = sample_rdp_settings();
        settings.scaling_mode = RDP_SCALING_MODE_PERCENTAGE.to_string();
        settings.scaling_percent = Some(125);
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP scaling percent must be 100, 140, or 180"
        );

        let mut settings = sample_rdp_settings();
        settings.scaling_mode = RDP_SCALING_MODE_PERCENTAGE.to_string();
        settings.scaling_percent = None;
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP scaling percent is required for percentage scaling"
        );

        let mut settings = sample_rdp_settings();
        settings.scaling_percent = Some(140);
        assert_eq!(
            build_rdp_argv("xfreerdp3", &sample_server(), &settings).unwrap_err(),
            "RDP scaling percent is only valid for percentage scaling"
        );
    }

    #[test]
    fn expands_home_in_identity_file_path() {
        let expanded =
            expand_home_path_with("~/.ssh/id_ed25519", Some(OsString::from("/home/alex")));

        assert_eq!(expanded, "/home/alex/.ssh/id_ed25519");
    }

    #[test]
    fn formats_copyable_ssh_command_with_quoting() {
        let command = format_argv_for_display(&[
            "ssh".to_string(),
            "-J".to_string(),
            "user@bastion:22".to_string(),
            "-i".to_string(),
            "/home/alex/.ssh/lab key".to_string(),
            "admin@nas.local".to_string(),
        ]);

        assert_eq!(
            command,
            "ssh -J user@bastion:22 -i '/home/alex/.ssh/lab key' admin@nas.local"
        );
    }

    #[test]
    fn builds_terminal_command_for_supported_terminals() {
        let ssh_argv = vec!["ssh".to_string(), "nas.local".to_string()];

        assert_eq!(
            terminal_command_for("konsole", &ssh_argv).unwrap(),
            ProcessCommand {
                program: "konsole".to_string(),
                args: vec!["--separate", "--noclose", "-e", "ssh", "nas.local"]
                    .into_iter()
                    .map(String::from)
                    .collect()
            }
        );
        assert_eq!(
            terminal_command_for("wezterm", &ssh_argv).unwrap().args,
            vec!["start", "--", "ssh", "nas.local"]
        );
        assert_eq!(
            terminal_command_for("alacritty", &ssh_argv).unwrap(),
            ProcessCommand {
                program: "alacritty".to_string(),
                args: vec!["-e", "ssh", "nas.local"]
                    .into_iter()
                    .map(String::from)
                    .collect()
            }
        );
        assert_eq!(
            terminal_command_for("gnome-terminal", &ssh_argv)
                .unwrap()
                .args,
            vec!["--", "ssh", "nas.local"]
        );
    }

    #[test]
    fn selects_auto_terminal_in_preferred_order() {
        let terminal = select_terminal(TERMINAL_PREFERENCE_AUTO, |candidate| {
            candidate == "alacritty" || candidate == "xterm"
        })
        .unwrap();

        assert_eq!(terminal, "alacritty");
    }

    #[test]
    fn explicit_missing_terminal_is_an_error() {
        assert_eq!(
            select_terminal("konsole", |_| false).unwrap_err(),
            "Preferred terminal 'Konsole' was not found in PATH"
        );
    }

    #[test]
    fn builds_terminal_test_command_with_selected_terminal() {
        let mut settings = sample_settings();
        settings.terminal_preference = "konsole".to_string();

        let command = build_terminal_test_command(&settings, |candidate| {
            candidate == "printf" || candidate == "konsole"
        })
        .unwrap();

        assert_eq!(command.program, "konsole");
        assert_eq!(command.args[0], "--separate");
        assert_eq!(command.args[1], "--noclose");
        assert_eq!(command.args[2], "-e");
        assert!(command.args.contains(&"printf".to_string()));
        assert!(command
            .args
            .contains(&"SSH-Buddy terminal test\n".to_string()));
    }

    #[test]
    fn selects_rdp_client_in_preferred_order() {
        let client =
            select_rdp_client(|candidate| candidate == "xfreerdp3" || candidate == "xfreerdp")
                .unwrap();
        assert_eq!(client, "xfreerdp3");

        let client = select_rdp_client(|candidate| candidate == "xfreerdp").unwrap();
        assert_eq!(client, "xfreerdp");

        assert_eq!(
            select_rdp_client(|_| false).unwrap_err(),
            "No supported RDP client found. Install FreeRDP xfreerdp3 or xfreerdp."
        );
    }

    #[test]
    fn launch_preflight_requires_ssh_binary() {
        assert_eq!(
            build_launch_command(&sample_server(), None, &sample_settings(), |_| false)
                .unwrap_err(),
            "OpenSSH client 'ssh' was not found in PATH. Install OpenSSH and try again."
        );
    }

    #[test]
    fn install_public_key_preflight_reports_missing_ssh_copy_id() {
        assert_eq!(
            build_install_public_key_launch_command(
                &sample_server(),
                "/home/user/.ssh/id_ed25519",
                &sample_settings(),
                |_| false
            )
            .unwrap_err(),
            "OpenSSH tool 'ssh-copy-id' was not found in PATH. Install the OpenSSH package that provides ssh-copy-id and try again."
        );
    }

    #[test]
    fn install_public_key_preflight_reports_missing_public_key() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("id_ed25519");

        let error = build_install_public_key_launch_command(
            &sample_server(),
            key_path.to_str().unwrap(),
            &sample_settings(),
            |candidate| candidate == "ssh-copy-id" || candidate == "konsole",
        )
        .unwrap_err();

        assert!(error.contains("Public key file not found"));
        assert!(error.contains("ssh-keygen -y -f"));
        assert!(error.contains(".pub"));
    }

    #[test]
    fn install_public_key_preflight_does_not_require_private_key_file() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("id_ed25519");
        fs::write(
            format!("{}.pub", key_path.to_string_lossy()),
            "ssh-ed25519 AAA test",
        )
        .unwrap();

        let command = build_install_public_key_launch_command(
            &sample_server(),
            key_path.to_str().unwrap(),
            &sample_settings(),
            |candidate| candidate == "ssh-copy-id" || candidate == "alacritty",
        )
        .unwrap();

        assert_eq!(command.program, "alacritty");
        assert!(command.args.contains(&"ssh-copy-id".to_string()));
        assert!(command
            .args
            .contains(&format!("{}.pub", key_path.to_string_lossy())));
    }

    #[test]
    fn tunnel_launch_preflight_reports_missing_ssh_and_terminal() {
        assert_eq!(
            build_tunnel_launch_command(
                &sample_server(),
                None,
                &sample_tunnel(),
                &sample_settings(),
                |_| false
            )
            .unwrap_err(),
            "OpenSSH client 'ssh' was not found in PATH. Install OpenSSH and try again."
        );

        let error = build_tunnel_launch_command(
            &sample_server(),
            None,
            &sample_tunnel(),
            &sample_settings(),
            |command| command == "ssh",
        )
        .unwrap_err();

        assert!(error.contains("No supported terminal found"));
    }

    #[test]
    fn sftp_launch_preflight_reports_missing_sftp_and_key_file() {
        assert_eq!(
            build_sftp_launch_command(&sample_server(), None, &sample_settings(), |_| false)
                .unwrap_err(),
            "OpenSSH SFTP client 'sftp' was not found in PATH. Install OpenSSH and try again."
        );

        let error = build_sftp_launch_command(
            &sample_server(),
            Some("/tmp/ssh-buddy-missing-test-key"),
            &sample_settings(),
            |command| command == "sftp" || command == "konsole",
        )
        .unwrap_err();

        assert!(error.contains("Selected SSH key file was not found"));
    }

    #[test]
    fn ssh_launch_diagnostics_include_command_key_and_binary_status() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("id_ed25519");
        let public_key_path = dir.path().join("id_ed25519.pub");
        fs::write(&key_path, "not a real key").unwrap();
        fs::write(&public_key_path, "not a real public key").unwrap();

        let (diagnostics, command) = build_ssh_launch_diagnostics(
            &sample_server(),
            Some(key_path.to_str().unwrap()),
            &sample_settings(),
            |candidate| candidate == "ssh" || candidate == "konsole",
        );

        assert!(command.is_some());
        assert_eq!(diagnostics.backend_result, "preflightPassed");
        assert_eq!(
            diagnostics.selected_terminal_or_client,
            Some("konsole".to_string())
        );
        assert!(diagnostics
            .executable
            .as_deref()
            .is_some_and(|value| value.ends_with("konsole")));
        assert!(diagnostics
            .argv_preview
            .as_deref()
            .is_some_and(|value| value.contains("--noclose -e ssh")));
        assert_eq!(diagnostics.key_file_exists, Some(true));
        assert_eq!(diagnostics.public_key_file_exists, Some(true));
        assert!(diagnostics.command_preview.starts_with("ssh -p 2222 -i "));
        assert!(diagnostics
            .required_binaries
            .iter()
            .any(|binary| binary.name == "ssh" && binary.exists));
    }

    #[test]
    fn install_public_key_launch_diagnostics_include_target_and_public_key() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("id_ed25519");
        let public_key_path = format!("{}.pub", key_path.to_string_lossy());
        fs::write(&public_key_path, "ssh-ed25519 AAA test").unwrap();

        let mut server = sample_server();
        server.proxy_jump = Some("jump.local".to_string());
        let (diagnostics, command) = build_install_public_key_launch_diagnostics(
            &server,
            key_path.to_str().unwrap(),
            &sample_settings(),
            |candidate| candidate == "ssh-copy-id" || candidate == "alacritty",
        );
        let command = command.unwrap();

        assert_eq!(command.program, "alacritty");
        assert_eq!(command.args[0], "-e");
        assert!(command.args.contains(&"ssh-copy-id".to_string()));
        assert!(command.args.contains(&public_key_path));
        assert_eq!(diagnostics.backend_result, "preflightPassed");
        assert_eq!(diagnostics.action_type, "install-public-key");
        assert_eq!(
            diagnostics.selected_terminal_or_client,
            Some("alacritty".to_string())
        );
        assert_eq!(diagnostics.key_file_exists, Some(false));
        assert_eq!(diagnostics.public_key_path, Some(public_key_path.clone()));
        assert_eq!(diagnostics.public_key_file_exists, Some(true));
        assert_eq!(diagnostics.target_username, Some("admin".to_string()));
        assert_eq!(diagnostics.target_host, Some("nas.local".to_string()));
        assert_eq!(diagnostics.target_port, Some(2222));
        assert_eq!(diagnostics.proxy_jump, Some("jump.local".to_string()));
        assert!(diagnostics.command_preview.starts_with("ssh-copy-id -i "));
        assert!(!command.args.iter().any(|arg| arg.contains("/p:")));
        assert!(diagnostics
            .required_binaries
            .iter()
            .any(|binary| binary.name == "ssh-copy-id" && binary.exists));
    }

    #[test]
    fn sftp_launch_diagnostics_report_missing_terminal() {
        let (diagnostics, command) = build_sftp_launch_diagnostics(
            &sample_server(),
            None,
            &sample_settings(),
            |candidate| candidate == "sftp",
        );

        assert!(command.is_none());
        assert_eq!(diagnostics.backend_result, "notFound");
        assert!(diagnostics.message.contains("No supported terminal found"));
        assert!(diagnostics.command_preview.starts_with("sftp -P 2222"));
        assert!(diagnostics
            .required_binaries
            .iter()
            .any(|binary| binary.name == "sftp" && binary.exists));
        assert!(diagnostics
            .required_binaries
            .iter()
            .any(|binary| binary.name == "konsole" && !binary.exists));
    }

    #[test]
    fn launch_diagnostics_report_missing_identity_file() {
        let missing_key = "/tmp/ssh-buddy-missing-test-key";
        let (diagnostics, command) = build_ssh_launch_diagnostics(
            &sample_server(),
            Some(missing_key),
            &sample_settings(),
            |candidate| candidate == "ssh" || candidate == "konsole",
        );

        assert!(command.is_none());
        assert_eq!(diagnostics.backend_result, "preflightFailed");
        assert!(diagnostics
            .message
            .contains("Selected SSH key file was not found"));
        assert_eq!(diagnostics.key_path, Some(missing_key.to_string()));
        assert_eq!(diagnostics.key_file_exists, Some(false));
        assert_eq!(
            diagnostics.public_key_path,
            Some(format!("{missing_key}.pub"))
        );
        assert_eq!(diagnostics.public_key_file_exists, Some(false));
    }

    #[test]
    fn rdp_launch_preflight_builds_process_command() {
        let command =
            build_rdp_launch_command(&sample_server(), &sample_rdp_settings(), |candidate| {
                candidate == "xfreerdp"
            })
            .unwrap();

        assert_eq!(command.program, "xfreerdp");
        assert_eq!(command.args[0], "/v:nas.local:3390");
        assert!(command.args.contains(&"/cert:tofu".to_string()));
        assert!(command.args.contains(&"/u:rdpuser".to_string()));
        assert!(command.args.contains(&"/monitors:0,1".to_string()));
        assert!(!command.args.iter().any(|arg| arg.starts_with("/p:")));
    }

    #[test]
    fn rdp_launch_diagnostics_wrap_free_rdp_in_terminal_for_prompts() {
        let (diagnostics, command) = build_rdp_launch_diagnostics(
            &sample_server(),
            &sample_rdp_settings(),
            &sample_settings(),
            |candidate| candidate == "xfreerdp" || candidate == "konsole",
        );
        let command = command.unwrap();

        assert_eq!(command.program, "konsole");
        assert_eq!(command.args[0], "--separate");
        assert_eq!(command.args[1], "--noclose");
        assert_eq!(command.args[2], "-e");
        assert!(command.args.contains(&"xfreerdp".to_string()));
        assert!(command.args.contains(&"/v:nas.local:3390".to_string()));
        assert!(command.args.contains(&"/cert:tofu".to_string()));
        assert_eq!(diagnostics.backend_result, "preflightPassed");
        assert_eq!(
            diagnostics.selected_terminal_or_client,
            Some("konsole -> xfreerdp".to_string())
        );
        assert!(diagnostics
            .executable
            .as_deref()
            .is_some_and(|value| value.ends_with("konsole")));
        assert!(diagnostics
            .argv_preview
            .as_deref()
            .is_some_and(|value| value.contains("--noclose -e xfreerdp")));
        assert!(diagnostics.command_preview.starts_with("xfreerdp "));
        assert_eq!(
            diagnostics.free_rdp_executable,
            Some("xfreerdp".to_string())
        );
        assert_eq!(diagnostics.launched_via_terminal, Some(true));
        assert_eq!(diagnostics.certificate_mode, Some("tofu".to_string()));
        assert_eq!(diagnostics.rdp_username, Some("rdpuser".to_string()));
        assert_eq!(diagnostics.rdp_domain, Some("LAB".to_string()));
        assert_eq!(diagnostics.rdp_port, Some(3390));
        assert_eq!(diagnostics.rdp_fullscreen, Some(false));
        assert_eq!(diagnostics.rdp_width, Some(1920));
        assert_eq!(diagnostics.rdp_height, Some(1080));
        assert_eq!(diagnostics.rdp_multi_monitor, Some(true));
        assert_eq!(diagnostics.rdp_monitor_ids, Some("0,1".to_string()));
        assert_eq!(
            diagnostics.rdp_scaling_mode,
            Some(RDP_SCALING_MODE_NATIVE.to_string())
        );
        assert_eq!(diagnostics.rdp_scaling_percent, None);
        assert_eq!(diagnostics.rdp_smart_sizing, Some(false));
        assert_eq!(diagnostics.rdp_dynamic_resolution, Some(false));
        assert!(diagnostics
            .message
            .contains("FreeRDP certificate and credential prompts"));
        assert!(diagnostics
            .required_binaries
            .iter()
            .any(|binary| binary.name == "xfreerdp" && binary.exists));
        assert!(diagnostics
            .required_binaries
            .iter()
            .any(|binary| binary.name == "konsole" && binary.exists));
        assert!(!command.args.iter().any(|arg| arg.starts_with("/p:")));
    }

    #[test]
    fn rdp_launch_diagnostics_report_missing_terminal() {
        let (diagnostics, command) = build_rdp_launch_diagnostics(
            &sample_server(),
            &sample_rdp_settings(),
            &sample_settings(),
            |candidate| candidate == "xfreerdp",
        );

        assert!(command.is_none());
        assert_eq!(diagnostics.backend_result, "notFound");
        assert!(diagnostics.message.contains("No supported terminal found"));
        assert!(diagnostics.command_preview.starts_with("xfreerdp "));
        assert!(diagnostics
            .required_binaries
            .iter()
            .any(|binary| binary.name == "xfreerdp" && binary.exists));
        assert!(diagnostics
            .required_binaries
            .iter()
            .any(|binary| binary.name == "konsole" && !binary.exists));
    }

    #[test]
    fn launch_preflight_rejects_missing_identity_file() {
        let error = build_launch_command(
            &sample_server(),
            Some("/tmp/ssh-buddy-missing-test-key"),
            &sample_settings(),
            |command| command == "ssh" || command == "konsole",
        )
        .unwrap_err();

        assert!(error.contains("Selected SSH key file was not found"));
    }

    #[test]
    fn launch_preflight_builds_process_command() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("id_ed25519");
        fs::write(&key_path, "not a real key").unwrap();

        let command = build_launch_command(
            &sample_server(),
            Some(key_path.to_str().unwrap()),
            &sample_settings(),
            |command| command == "ssh" || command == "konsole",
        )
        .unwrap();

        assert_eq!(command.program, "konsole");
        assert_eq!(command.args[0], "--separate");
        assert_eq!(command.args[1], "--noclose");
        assert_eq!(command.args[2], "-e");
        assert!(command.args.contains(&"ssh".to_string()));
        assert!(command
            .args
            .contains(&key_path.to_string_lossy().into_owned()));
    }

    #[test]
    fn terminal_availability_reports_absolute_executable_paths() {
        let availability = terminal_availability_with(|terminal| {
            (terminal == "konsole").then(|| PathBuf::from("/usr/bin/konsole"))
        });

        let konsole = availability
            .iter()
            .find(|terminal| terminal.preference == "konsole")
            .unwrap();
        assert!(konsole.available);
        assert_eq!(konsole.executable.as_deref(), Some("/usr/bin/konsole"));
        assert!(
            !availability
                .iter()
                .find(|terminal| terminal.preference == "alacritty")
                .unwrap()
                .available
        );
    }

    #[test]
    fn non_appimage_environment_is_preserved() {
        let environment = host_launch_environment_from([
            (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
            (OsString::from("DISPLAY"), OsString::from(":1")),
            (
                OsString::from("DBUS_SESSION_BUS_ADDRESS"),
                OsString::from("unix:path=/run/user/1000/bus"),
            ),
        ]);

        assert!(!environment.sanitized);
        assert_eq!(
            environment.variables.get(OsStr::new("DISPLAY")),
            Some(&OsString::from(":1"))
        );
        assert_eq!(environment.variables.len(), 3);
    }

    #[test]
    fn appimage_environment_restores_host_paths_and_filters_both_preload_delimiters() {
        let appdir = "/tmp/.mount_SSHBud123";
        for preload in [
            format!("/usr/lib/libhost-first.so:{appdir}/usr/lib/libappimage.so:/opt/lib/libhost-last.so"),
            format!("{appdir}/usr/lib/libappimage.so:/usr/lib/libhost-first.so:/opt/lib/libhost-last.so"),
            format!("/usr/lib/libhost-first.so {appdir}/usr/lib/libappimage.so /opt/lib/libhost-last.so"),
            format!("{appdir}/usr/lib/libappimage.so /usr/lib/libhost-first.so /opt/lib/libhost-last.so"),
        ] {
            let environment = host_launch_environment_from([
                (OsString::from("APPIMAGE"), OsString::from("/opt/SSH-Buddy.AppImage")),
                (OsString::from("APPDIR"), OsString::from(appdir)),
                (
                    OsString::from("PATH"),
                    OsString::from(format!("{appdir}/usr/bin:/appimage/fallback")),
                ),
                (
                    OsString::from("PATH_ORIG"),
                    OsString::from("/usr/local/bin:/usr/bin:/bin"),
                ),
                (
                    OsString::from("LD_LIBRARY_PATH"),
                    OsString::from(format!("{appdir}/usr/lib:/appimage/fallback")),
                ),
                (
                    OsString::from("LD_LIBRARY_PATH_ORIG"),
                    OsString::from("/usr/local/lib:/usr/lib"),
                ),
                (OsString::from("LD_PRELOAD"), OsString::from(preload)),
                (
                    OsString::from("QT_PLUGIN_PATH"),
                    OsString::from(format!("{appdir}/usr/lib/qt6/plugins")),
                ),
                (OsString::from("WAYLAND_DISPLAY"), OsString::from("wayland-0")),
                (OsString::from("DISPLAY"), OsString::from(":0")),
                (
                    OsString::from("DBUS_SESSION_BUS_ADDRESS"),
                    OsString::from("unix:path=/run/user/1000/bus"),
                ),
                (
                    OsString::from("SSH_AUTH_SOCK"),
                    OsString::from("/run/user/1000/ssh-agent.socket"),
                ),
                (OsString::from("LANG"), OsString::from("en_US.UTF-8")),
                (OsString::from("LC_ALL"), OsString::from("en_US.UTF-8")),
            ]);

            assert!(environment.sanitized);
            assert_eq!(
                environment.variables.get(OsStr::new("PATH")),
                Some(&OsString::from("/usr/local/bin:/usr/bin:/bin"))
            );
            assert_eq!(
                environment.variables.get(OsStr::new("LD_LIBRARY_PATH")),
                Some(&OsString::from("/usr/local/lib:/usr/lib"))
            );
            assert_eq!(
                environment.variables.get(OsStr::new("LD_PRELOAD")),
                Some(&OsString::from(
                    "/usr/lib/libhost-first.so:/opt/lib/libhost-last.so"
                ))
            );
            assert_eq!(
                command_path_in_with(
                    "konsole",
                    environment.variables.get(OsStr::new("PATH")),
                    |path| path == Path::new("/usr/bin/konsole"),
                ),
                Some(PathBuf::from("/usr/bin/konsole"))
            );
            for name in [
                "WAYLAND_DISPLAY",
                "DISPLAY",
                "DBUS_SESSION_BUS_ADDRESS",
                "SSH_AUTH_SOCK",
                "LANG",
                "LC_ALL",
            ] {
                assert!(environment.variables.contains_key(OsStr::new(name)));
            }
            for value in environment.variables.values() {
                assert!(!value.to_string_lossy().contains(".mount_"));
            }
            for name in [
                "APPDIR",
                "APPIMAGE",
                "PATH_ORIG",
                "LD_LIBRARY_PATH_ORIG",
                "QT_PLUGIN_PATH",
            ] {
                assert!(!environment.variables.contains_key(OsStr::new(name)));
            }
        }
    }

    #[test]
    fn classifies_fake_immediate_nonzero_exit_with_bounded_stderr() {
        let mut settings = sample_settings();
        settings.terminal_preference = "konsole".to_string();
        let (diagnostics, _) = build_terminal_test_diagnostics(&settings, |candidate| {
            candidate == "printf" || candidate == "konsole"
        });
        let command = ProcessCommand {
            program: env::current_exe().unwrap().to_string_lossy().into_owned(),
            args: Vec::new(),
        };
        let environment = HostLaunchEnvironment {
            variables: BTreeMap::new(),
            sanitized: true,
        };
        let runner = FakeProcessRunner {
            observation: ProcessObservation::Exited {
                code: Some(23),
                stderr: format!("loader failure\u{7} {}", "x".repeat(3_000)),
            },
        };

        let diagnostics =
            spawn_with_diagnostics_using(diagnostics, Some(command), &runner, &environment);

        assert_eq!(diagnostics.backend_result, "exitedImmediately");
        assert_eq!(diagnostics.exit_code, Some(23));
        assert!(diagnostics
            .stderr
            .as_deref()
            .unwrap()
            .starts_with("loader failure"));
        assert!(diagnostics.stderr.as_deref().unwrap().len() <= STDERR_LIMIT_BYTES);
        assert!(!diagnostics.stderr.as_deref().unwrap().contains('\u{7}'));
        assert!(diagnostics.environment_sanitized);
    }

    #[test]
    fn classifies_running_and_cleanly_detached_fake_processes_as_started() {
        for observation in [
            ProcessObservation::Running,
            ProcessObservation::Exited {
                code: Some(0),
                stderr: String::new(),
            },
        ] {
            let (diagnostics, _) =
                build_terminal_test_diagnostics(&sample_settings(), |candidate| {
                    candidate == "printf" || candidate == "konsole"
                });
            let command = ProcessCommand {
                program: env::current_exe().unwrap().to_string_lossy().into_owned(),
                args: Vec::new(),
            };
            let environment = HostLaunchEnvironment {
                variables: BTreeMap::new(),
                sanitized: false,
            };
            let runner = FakeProcessRunner { observation };

            let diagnostics =
                spawn_with_diagnostics_using(diagnostics, Some(command), &runner, &environment);

            assert_eq!(diagnostics.backend_result, "started");
        }
    }

    #[test]
    fn classifies_missing_and_spawn_failed_launches() {
        let (diagnostics, _) = build_terminal_test_diagnostics(&sample_settings(), |candidate| {
            candidate == "printf" || candidate == "konsole"
        });
        let missing = spawn_with_diagnostics_using(
            diagnostics.clone(),
            Some(ProcessCommand {
                program: "ssh-buddy-terminal-that-does-not-exist".to_string(),
                args: Vec::new(),
            }),
            &FakeProcessRunner {
                observation: ProcessObservation::Running,
            },
            &HostLaunchEnvironment {
                variables: BTreeMap::new(),
                sanitized: false,
            },
        );
        assert_eq!(missing.backend_result, "notFound");

        let failed = spawn_with_diagnostics_using(
            diagnostics,
            Some(ProcessCommand {
                program: env::current_exe().unwrap().to_string_lossy().into_owned(),
                args: Vec::new(),
            }),
            &FakeProcessRunner {
                observation: ProcessObservation::SpawnFailed("permission denied".to_string()),
            },
            &HostLaunchEnvironment {
                variables: BTreeMap::new(),
                sanitized: false,
            },
        );
        assert_eq!(failed.backend_result, "spawnFailed");
        assert!(failed.message.contains("permission denied"));
    }
}
