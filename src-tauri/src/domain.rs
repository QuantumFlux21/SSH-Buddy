use std::collections::HashSet;

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

pub type AppResult<T> = Result<T, String>;

pub const TERMINAL_PREFERENCE_AUTO: &str = "auto";
pub const SUPPORTED_TERMINAL_PREFERENCES: &[&str] = &[
    TERMINAL_PREFERENCE_AUTO,
    "konsole",
    "kitty",
    "alacritty",
    "wezterm",
    "gnome-terminal",
    "xterm",
];
pub const TUNNEL_TYPE_LOCAL: &str = "local";
pub const DEFAULT_LOCAL_BIND_HOST: &str = "127.0.0.1";
pub const DEFAULT_RDP_PORT: u16 = 3389;
pub const SUPPORTED_RDP_COLOR_DEPTHS: &[u16] = &[16, 24, 32];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SshKeyRef {
    pub id: String,
    pub label: String,
    pub path: String,
    pub fingerprint: Option<String>,
    pub comment: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebLink {
    pub id: String,
    pub server_profile_id: String,
    pub label: String,
    pub url: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tunnel {
    pub id: String,
    pub server_profile_id: String,
    pub label: String,
    pub tunnel_type: String,
    pub local_bind_host: Option<String>,
    pub local_port: Option<u16>,
    pub remote_host: Option<String>,
    pub remote_port: Option<u16>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RdpSettings {
    pub server_profile_id: String,
    pub enabled: bool,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub port: u16,
    pub fullscreen: bool,
    pub multi_monitor: bool,
    pub monitor_ids: Option<String>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub color_depth: Option<u16>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerProfile {
    pub id: String,
    pub display_name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub identity_file_id: Option<String>,
    pub proxy_jump: Option<String>,
    pub group_id: Option<String>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub tags: Vec<Tag>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub terminal_preference: String,
    pub safety_warnings_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppStateSnapshot {
    pub servers: Vec<ServerProfile>,
    pub groups: Vec<Group>,
    pub tags: Vec<Tag>,
    pub ssh_keys: Vec<SshKeyRef>,
    pub settings: AppSettings,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInput {
    pub display_name: String,
    pub host: String,
    pub port: u32,
    pub username: String,
    pub identity_file_id: Option<String>,
    pub proxy_jump: Option<String>,
    pub group_id: Option<String>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub tag_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInput {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshKeyInput {
    pub label: String,
    pub path: String,
    pub fingerprint: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebLinkInput {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInput {
    pub label: String,
    pub tunnel_type: String,
    pub local_bind_host: Option<String>,
    pub local_port: Option<u32>,
    pub remote_host: Option<String>,
    pub remote_port: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RdpSettingsInput {
    pub enabled: bool,
    pub username: Option<String>,
    pub domain: Option<String>,
    pub port: Option<u32>,
    pub fullscreen: bool,
    pub multi_monitor: bool,
    pub monitor_ids: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub color_depth: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCandidate {
    pub alias: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub identity_file: Option<String>,
    pub proxy_jump: Option<String>,
    pub warnings: Vec<String>,
    pub selected: bool,
    pub duplicate: bool,
    pub skipped: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub servers: Vec<ServerProfile>,
}

pub fn default_settings() -> AppSettings {
    AppSettings {
        terminal_preference: TERMINAL_PREFERENCE_AUTO.to_string(),
        safety_warnings_enabled: true,
    }
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn now_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|item| normalize_text(&item))
}

pub fn normalize_text(value: &str) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub fn normalize_notes(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub fn normalize_tag_names(tag_names: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for tag_name in tag_names {
        if let Some(name) = normalize_text(tag_name) {
            let key = name.to_lowercase();
            if seen.insert(key) {
                normalized.push(name);
            }
        }
    }

    normalized
}

pub fn validate_server_input(input: &ServerInput) -> AppResult<()> {
    if normalize_text(&input.display_name).is_none() {
        return Err("Display name is required".to_string());
    }

    if normalize_text(&input.host).is_none() {
        return Err("Host is required".to_string());
    }

    if input.port == 0 || input.port > 65_535 {
        return Err("Port must be between 1 and 65535".to_string());
    }

    if let Some(proxy_jump) = &input.proxy_jump {
        validate_proxy_jump(proxy_jump)?;
    }

    Ok(())
}

pub fn validate_proxy_jump(value: &str) -> AppResult<()> {
    let proxy_jump = value.trim();
    if proxy_jump.is_empty() {
        return Err("ProxyJump cannot be blank".to_string());
    }

    if proxy_jump.chars().any(char::is_whitespace) {
        return Err("ProxyJump must not contain whitespace".to_string());
    }

    if proxy_jump
        .chars()
        .any(|character| !is_proxy_jump_character_allowed(character))
    {
        return Err(
            "ProxyJump contains unsupported characters. Use OpenSSH host specs like user@bastion:22."
                .to_string(),
        );
    }

    for jump in proxy_jump.split(',') {
        validate_proxy_jump_entry(jump)?;
    }

    Ok(())
}

fn validate_proxy_jump_entry(entry: &str) -> AppResult<()> {
    if entry.is_empty() {
        return Err("ProxyJump entries must not be empty".to_string());
    }

    let mut parts = entry.split('@');
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    if parts.next().is_some() {
        return Err("ProxyJump entries may contain at most one @".to_string());
    }

    let host = if let Some(host) = second {
        if first.is_empty() {
            return Err("ProxyJump username must not be empty".to_string());
        }
        host
    } else {
        first
    };

    if host.is_empty() {
        return Err("ProxyJump host must not be empty".to_string());
    }

    if host.starts_with('-') {
        return Err("ProxyJump host must not start with '-'".to_string());
    }

    validate_proxy_jump_host_and_port(host)
}

fn validate_proxy_jump_host_and_port(host: &str) -> AppResult<()> {
    if let Some(remainder) = host.strip_prefix('[') {
        let Some(end) = remainder.find(']') else {
            return Err("ProxyJump IPv6 hosts must close ']'".to_string());
        };
        let address = &remainder[..end];
        let suffix = &remainder[end + 1..];
        if address.is_empty() {
            return Err("ProxyJump host must not be empty".to_string());
        }
        if !suffix.is_empty() {
            let Some(port) = suffix.strip_prefix(':') else {
                return Err("ProxyJump IPv6 host suffix must be a port like :22".to_string());
            };
            validate_proxy_jump_port(port)?;
        }
        return Ok(());
    }

    let colon_count = host.chars().filter(|character| *character == ':').count();
    if colon_count == 1 {
        let (host_part, port_part) = host.rsplit_once(':').unwrap_or((host, ""));
        if host_part.is_empty() {
            return Err("ProxyJump host must not be empty".to_string());
        }
        validate_proxy_jump_port(port_part)?;
    }

    Ok(())
}

fn validate_proxy_jump_port(port: &str) -> AppResult<()> {
    match port.parse::<u16>() {
        Ok(value) if value > 0 => Ok(()),
        _ => Err("ProxyJump port must be between 1 and 65535".to_string()),
    }
}

fn is_proxy_jump_character_allowed(character: char) -> bool {
    character.is_ascii_alphanumeric() || "._-+@:%[],".contains(character)
}

pub fn validate_group_input(input: &GroupInput) -> AppResult<()> {
    if normalize_text(&input.name).is_none() {
        return Err("Group name is required".to_string());
    }

    Ok(())
}

pub fn validate_ssh_key_input(input: &SshKeyInput) -> AppResult<()> {
    if normalize_text(&input.label).is_none() {
        return Err("Key label is required".to_string());
    }

    if normalize_text(&input.path).is_none() {
        return Err("Key path is required".to_string());
    }

    Ok(())
}

pub fn validate_web_link_input(input: &WebLinkInput) -> AppResult<()> {
    if normalize_text(&input.label).is_none() {
        return Err("Web link label is required".to_string());
    }

    validate_web_link_url(&input.url)?;

    Ok(())
}

pub fn validate_web_link_url(value: &str) -> AppResult<()> {
    let url = value.trim();
    if url.is_empty() {
        return Err("Web link URL is required".to_string());
    }

    let parsed = Url::parse(url).map_err(|_| "Web link URL must be a valid URL".to_string())?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => return Err("Web link URL must start with http:// or https://".to_string()),
    }

    if parsed.host_str().is_none() {
        return Err("Web link URL must include a host".to_string());
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("Web link URL must not include embedded credentials".to_string());
    }

    Ok(())
}

pub fn validate_tunnel_input(input: &TunnelInput) -> AppResult<()> {
    if normalize_text(&input.label).is_none() {
        return Err("Tunnel label is required".to_string());
    }

    if input.tunnel_type.trim() != TUNNEL_TYPE_LOCAL {
        return Err("Only local SSH tunnels are supported right now".to_string());
    }

    if let Some(local_bind_host) = &input.local_bind_host {
        let trimmed = local_bind_host.trim();
        if !trimmed.is_empty() {
            validate_tunnel_host(trimmed, "Local bind host")?;
        }
    }

    validate_tunnel_port(input.local_port, "Local port")?;

    let remote_host = input.remote_host.as_deref().unwrap_or_default().trim();
    if remote_host.is_empty() {
        return Err("Remote host is required".to_string());
    }
    validate_tunnel_host(remote_host, "Remote host")?;

    validate_tunnel_port(input.remote_port, "Remote port")?;

    Ok(())
}

pub fn normalize_tunnel_bind_host(value: Option<String>) -> String {
    value
        .and_then(|item| normalize_text(&item))
        .unwrap_or_else(|| DEFAULT_LOCAL_BIND_HOST.to_string())
}

pub fn validate_rdp_settings_input(input: &RdpSettingsInput) -> AppResult<()> {
    validate_rdp_port(input.port.unwrap_or(u32::from(DEFAULT_RDP_PORT)))?;

    if let Some(username) = &input.username {
        validate_rdp_text_field(username, "RDP username")?;
    }

    if let Some(domain) = &input.domain {
        validate_rdp_text_field(domain, "RDP domain")?;
    }

    if let Some(monitor_ids) = &input.monitor_ids {
        validate_rdp_monitor_ids(monitor_ids)?;
        if normalize_text(monitor_ids).is_some() && !input.multi_monitor {
            return Err("RDP monitor IDs require multi-monitor".to_string());
        }
    }

    match (input.width, input.height) {
        (None, None) => {}
        (Some(width), Some(height)) => {
            validate_rdp_dimension(width, "RDP width")?;
            validate_rdp_dimension(height, "RDP height")?;
        }
        _ => return Err("RDP width and height must be set together".to_string()),
    }

    if let Some(depth) = input.color_depth {
        if !SUPPORTED_RDP_COLOR_DEPTHS.contains(&(depth as u16)) {
            return Err("RDP color depth must be 16, 24, or 32".to_string());
        }
    }

    Ok(())
}

pub fn normalize_rdp_monitor_ids(value: Option<String>) -> Option<String> {
    value.and_then(|item| normalize_text(&item))
}

fn validate_rdp_port(port: u32) -> AppResult<()> {
    if (1..=65_535).contains(&port) {
        Ok(())
    } else {
        Err("RDP port must be between 1 and 65535".to_string())
    }
}

fn validate_rdp_dimension(value: u32, label: &str) -> AppResult<()> {
    if (320..=16_384).contains(&value) {
        Ok(())
    } else {
        Err(format!("{label} must be between 320 and 16384"))
    }
}

pub fn validate_rdp_monitor_ids(value: &str) -> AppResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    if trimmed != value || trimmed.chars().any(char::is_whitespace) {
        return Err("RDP monitor IDs must not contain whitespace".to_string());
    }

    for entry in trimmed.split(',') {
        if entry.is_empty() {
            return Err("RDP monitor IDs must not contain empty entries".to_string());
        }

        if !entry.chars().all(|character| character.is_ascii_digit()) {
            return Err("RDP monitor IDs must be comma-separated monitor numbers".to_string());
        }
    }

    Ok(())
}

fn validate_rdp_text_field(value: &str, label: &str) -> AppResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    if trimmed.chars().any(char::is_control) {
        return Err(format!("{label} must not contain control characters"));
    }

    Ok(())
}

fn validate_tunnel_port(value: Option<u32>, label: &str) -> AppResult<()> {
    match value {
        Some(port) if (1..=65_535).contains(&port) => Ok(()),
        _ => Err(format!("{label} must be between 1 and 65535")),
    }
}

fn validate_tunnel_host(value: &str, label: &str) -> AppResult<()> {
    if value.chars().any(char::is_whitespace) {
        return Err(format!("{label} must not contain whitespace"));
    }

    if value.starts_with('-') {
        return Err(format!("{label} must not start with '-'"));
    }

    if value
        .chars()
        .any(|character| !is_tunnel_host_character_allowed(character))
    {
        return Err(format!(
            "{label} contains unsupported characters. Use a hostname or IP address."
        ));
    }

    Ok(())
}

fn is_tunnel_host_character_allowed(character: char) -> bool {
    character.is_ascii_alphanumeric() || "._-".contains(character)
}

pub fn validate_app_settings(input: &AppSettings) -> AppResult<()> {
    if !SUPPORTED_TERMINAL_PREFERENCES.contains(&input.terminal_preference.as_str()) {
        return Err("Unsupported terminal preference".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_server_input() -> ServerInput {
        ServerInput {
            display_name: "NAS".to_string(),
            host: "nas.local".to_string(),
            port: 22,
            username: "admin".to_string(),
            identity_file_id: None,
            proxy_jump: None,
            group_id: None,
            notes: None,
            favorite: false,
            tag_names: vec![],
        }
    }

    #[test]
    fn validates_server_required_fields() {
        let mut input = valid_server_input();
        input.display_name = " ".to_string();
        assert_eq!(
            validate_server_input(&input).unwrap_err(),
            "Display name is required"
        );

        let mut input = valid_server_input();
        input.host = " ".to_string();
        assert_eq!(
            validate_server_input(&input).unwrap_err(),
            "Host is required"
        );

        let mut input = valid_server_input();
        input.port = 0;
        assert_eq!(
            validate_server_input(&input).unwrap_err(),
            "Port must be between 1 and 65535"
        );

        let mut input = valid_server_input();
        input.port = 65_536;
        assert_eq!(
            validate_server_input(&input).unwrap_err(),
            "Port must be between 1 and 65535"
        );
    }

    #[test]
    fn validates_proxy_jump_values() {
        for proxy_jump in [
            "bastion",
            "user@bastion",
            "user@bastion:22",
            "user@bastion:22,jump2",
            "user@[2001:db8::1]:2222",
        ] {
            let mut input = valid_server_input();
            input.proxy_jump = Some(proxy_jump.to_string());
            assert!(validate_server_input(&input).is_ok(), "{proxy_jump}");
        }

        for (proxy_jump, message) in [
            (" ", "ProxyJump cannot be blank"),
            ("bastion;touch", "ProxyJump contains unsupported characters. Use OpenSSH host specs like user@bastion:22."),
            ("bastion proxy", "ProxyJump must not contain whitespace"),
            ("bastion,", "ProxyJump entries must not be empty"),
            ("user@@bastion", "ProxyJump entries may contain at most one @"),
            ("@bastion", "ProxyJump username must not be empty"),
            ("-bastion", "ProxyJump host must not start with '-'"),
            ("bastion:0", "ProxyJump port must be between 1 and 65535"),
        ] {
            let mut input = valid_server_input();
            input.proxy_jump = Some(proxy_jump.to_string());
            assert_eq!(validate_server_input(&input).unwrap_err(), message);
        }
    }

    fn valid_tunnel_input() -> TunnelInput {
        TunnelInput {
            label: "Postgres".to_string(),
            tunnel_type: TUNNEL_TYPE_LOCAL.to_string(),
            local_bind_host: Some("127.0.0.1".to_string()),
            local_port: Some(15432),
            remote_host: Some("db.internal".to_string()),
            remote_port: Some(5432),
        }
    }

    fn valid_rdp_settings_input() -> RdpSettingsInput {
        RdpSettingsInput {
            enabled: true,
            username: Some("labuser".to_string()),
            domain: Some("LAB".to_string()),
            port: Some(3389),
            fullscreen: false,
            multi_monitor: true,
            monitor_ids: Some("0,1".to_string()),
            width: Some(1920),
            height: Some(1080),
            color_depth: Some(32),
        }
    }

    #[test]
    fn validates_tunnel_input() {
        assert!(validate_tunnel_input(&valid_tunnel_input()).is_ok());

        let mut input = valid_tunnel_input();
        input.local_bind_host = None;
        assert!(validate_tunnel_input(&input).is_ok());
        assert_eq!(normalize_tunnel_bind_host(None), DEFAULT_LOCAL_BIND_HOST);

        let mut input = valid_tunnel_input();
        input.label = " ".to_string();
        assert_eq!(
            validate_tunnel_input(&input).unwrap_err(),
            "Tunnel label is required"
        );

        let mut input = valid_tunnel_input();
        input.tunnel_type = "remote".to_string();
        assert_eq!(
            validate_tunnel_input(&input).unwrap_err(),
            "Only local SSH tunnels are supported right now"
        );

        let mut input = valid_tunnel_input();
        input.local_port = Some(0);
        assert_eq!(
            validate_tunnel_input(&input).unwrap_err(),
            "Local port must be between 1 and 65535"
        );

        let mut input = valid_tunnel_input();
        input.remote_port = Some(65_536);
        assert_eq!(
            validate_tunnel_input(&input).unwrap_err(),
            "Remote port must be between 1 and 65535"
        );

        let mut input = valid_tunnel_input();
        input.remote_host = Some("db internal".to_string());
        assert_eq!(
            validate_tunnel_input(&input).unwrap_err(),
            "Remote host must not contain whitespace"
        );

        let mut input = valid_tunnel_input();
        input.remote_host = Some("db;touch".to_string());
        assert_eq!(
            validate_tunnel_input(&input).unwrap_err(),
            "Remote host contains unsupported characters. Use a hostname or IP address."
        );
    }

    #[test]
    fn validates_rdp_settings_input() {
        assert!(validate_rdp_settings_input(&valid_rdp_settings_input()).is_ok());

        let mut input = valid_rdp_settings_input();
        input.port = Some(0);
        assert_eq!(
            validate_rdp_settings_input(&input).unwrap_err(),
            "RDP port must be between 1 and 65535"
        );

        let mut input = valid_rdp_settings_input();
        input.width = Some(1920);
        input.height = None;
        assert_eq!(
            validate_rdp_settings_input(&input).unwrap_err(),
            "RDP width and height must be set together"
        );

        let mut input = valid_rdp_settings_input();
        input.width = Some(100);
        input.height = Some(1080);
        assert_eq!(
            validate_rdp_settings_input(&input).unwrap_err(),
            "RDP width must be between 320 and 16384"
        );

        let mut input = valid_rdp_settings_input();
        input.color_depth = Some(8);
        assert_eq!(
            validate_rdp_settings_input(&input).unwrap_err(),
            "RDP color depth must be 16, 24, or 32"
        );

        let mut input = valid_rdp_settings_input();
        input.monitor_ids = Some("0, 1".to_string());
        assert_eq!(
            validate_rdp_settings_input(&input).unwrap_err(),
            "RDP monitor IDs must not contain whitespace"
        );

        let mut input = valid_rdp_settings_input();
        input.monitor_ids = Some("0,,1".to_string());
        assert_eq!(
            validate_rdp_settings_input(&input).unwrap_err(),
            "RDP monitor IDs must not contain empty entries"
        );

        let mut input = valid_rdp_settings_input();
        input.monitor_ids = Some("-1".to_string());
        assert_eq!(
            validate_rdp_settings_input(&input).unwrap_err(),
            "RDP monitor IDs must be comma-separated monitor numbers"
        );

        let mut input = valid_rdp_settings_input();
        input.multi_monitor = false;
        assert_eq!(
            validate_rdp_settings_input(&input).unwrap_err(),
            "RDP monitor IDs require multi-monitor"
        );
    }

    #[test]
    fn normalizes_tags_case_insensitively() {
        let tags = normalize_tag_names(&[
            " prod ".to_string(),
            "PROD".to_string(),
            "linux".to_string(),
            "".to_string(),
        ]);

        assert_eq!(tags, vec!["prod".to_string(), "linux".to_string()]);
    }
}
