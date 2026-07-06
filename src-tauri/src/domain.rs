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
pub struct ServerProfile {
    pub id: String,
    pub display_name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub identity_file_id: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCandidate {
    pub alias: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub identity_file_id: Option<String>,
    pub warnings: Vec<String>,
    pub selected: bool,
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

    Ok(())
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
