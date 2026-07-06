use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::domain::{
    AppResult, AppSettings, ServerProfile, SUPPORTED_TERMINAL_PREFERENCES, TERMINAL_PREFERENCE_AUTO,
};

const TERMINAL_ORDER: &[&str] = &[
    "konsole",
    "kitty",
    "alacritty",
    "wezterm",
    "gnome-terminal",
    "xterm",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessCommand {
    pub program: String,
    pub args: Vec<String>,
}

pub fn build_ssh_argv(
    server: &ServerProfile,
    identity_file: Option<&str>,
) -> AppResult<Vec<String>> {
    if server.host.trim().is_empty() {
        return Err("Server profile is missing a host".to_string());
    }

    if server.port == 0 {
        return Err("Server profile port must be between 1 and 65535".to_string());
    }

    let mut argv = vec!["ssh".to_string()];
    if server.port != 22 {
        argv.push("-p".to_string());
        argv.push(server.port.to_string());
    }

    if let Some(identity_file) = identity_file.and_then(normalize_optional) {
        argv.push("-i".to_string());
        argv.push(expand_home_path(&identity_file));
    }

    let destination = if server.username.trim().is_empty() {
        server.host.trim().to_string()
    } else {
        format!("{}@{}", server.username.trim(), server.host.trim())
    };
    argv.push(destination);

    Ok(argv)
}

pub fn format_argv_for_display(argv: &[String]) -> String {
    argv.iter()
        .map(|part| shell_quote(part))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn terminal_command_for(terminal: &str, ssh_argv: &[String]) -> AppResult<ProcessCommand> {
    if ssh_argv.is_empty() {
        return Err("SSH command is empty".to_string());
    }

    let mut args = match terminal {
        "konsole" => vec!["-e".to_string()],
        "kitty" => Vec::new(),
        "alacritty" => vec!["-e".to_string()],
        "wezterm" => vec!["start".to_string(), "--".to_string()],
        "gnome-terminal" => vec!["--".to_string()],
        "xterm" => vec!["-e".to_string()],
        _ => return Err("Unsupported terminal preference".to_string()),
    };
    args.extend(ssh_argv.iter().cloned());

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
) -> AppResult<()> {
    let command = build_launch_command(server, identity_file, settings, command_in_path)?;
    let terminal = command.program.clone();

    Command::new(&command.program)
        .args(&command.args)
        .spawn()
        .map_err(|error| format!("Failed to launch {}: {error}", terminal_label(&terminal)))?;

    Ok(())
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

fn command_in_path(command: &str) -> bool {
    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path_var).any(|path| {
        let candidate = path.join(command);
        is_executable_file(&candidate)
    })
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

    fn sample_server() -> ServerProfile {
        ServerProfile {
            id: "srv".to_string(),
            display_name: "NAS".to_string(),
            host: "nas.local".to_string(),
            port: 2222,
            username: "admin".to_string(),
            identity_file_id: None,
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
        }
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
        server.username = " ".to_string();

        let argv = build_ssh_argv(&server, None).unwrap();

        assert_eq!(argv, vec!["ssh", "nas.local"]);
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
            "-i".to_string(),
            "/home/alex/.ssh/lab key".to_string(),
            "admin@nas.local".to_string(),
        ]);

        assert_eq!(command, "ssh -i '/home/alex/.ssh/lab key' admin@nas.local");
    }

    #[test]
    fn builds_terminal_command_for_supported_terminals() {
        let ssh_argv = vec!["ssh".to_string(), "nas.local".to_string()];

        assert_eq!(
            terminal_command_for("konsole", &ssh_argv).unwrap(),
            ProcessCommand {
                program: "konsole".to_string(),
                args: vec!["-e", "ssh", "nas.local"]
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
    fn launch_preflight_requires_ssh_binary() {
        assert_eq!(
            build_launch_command(&sample_server(), None, &sample_settings(), |_| false)
                .unwrap_err(),
            "OpenSSH client 'ssh' was not found in PATH. Install OpenSSH and try again."
        );
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
        assert_eq!(command.args[0], "-e");
        assert!(command.args.contains(&"ssh".to_string()));
        assert!(command
            .args
            .contains(&key_path.to_string_lossy().into_owned()));
    }
}
