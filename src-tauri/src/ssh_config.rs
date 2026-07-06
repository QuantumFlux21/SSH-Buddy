use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    db::Database,
    domain::{AppResult, ImportCandidate, ImportResult, ServerInput, ServerProfile},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedHostConfig {
    pub host: Option<String>,
    pub username: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
    pub proxy_jump: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedHost {
    alias: String,
    host: Option<String>,
    username: Option<String>,
    port: Option<u16>,
    identity_file: Option<String>,
    proxy_jump: Option<String>,
    warnings: Vec<String>,
    skipped: bool,
}

#[derive(Debug, Clone, Default)]
struct HostBlock {
    patterns: Vec<String>,
    host: Option<String>,
    username: Option<String>,
    port: Option<u16>,
    identity_file: Option<String>,
    proxy_jump: Option<String>,
    warnings: Vec<String>,
}

type Resolver = dyn Fn(&str) -> AppResult<Option<ResolvedHostConfig>>;

pub fn default_ssh_config_path() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".ssh/config"))
}

pub fn import_preview(db: &Database) -> AppResult<Vec<ImportCandidate>> {
    let Some(path) = default_ssh_config_path() else {
        return Ok(Vec::new());
    };
    import_preview_from_path(db, &path, &resolve_with_ssh_g)
}

pub fn import_selected(db: &Database, aliases: Vec<String>) -> AppResult<ImportResult> {
    let Some(path) = default_ssh_config_path() else {
        return Ok(ImportResult {
            imported: 0,
            skipped: aliases.len(),
            servers: Vec::new(),
        });
    };
    import_selected_from_path(db, &path, aliases, &resolve_with_ssh_g)
}

fn import_preview_from_path(
    db: &Database,
    path: &Path,
    resolver: &Resolver,
) -> AppResult<Vec<ImportCandidate>> {
    let parsed = read_and_parse(path)?;
    let existing_servers = db.list_servers()?;
    Ok(build_candidates(parsed, &existing_servers, resolver))
}

fn import_selected_from_path(
    db: &Database,
    path: &Path,
    aliases: Vec<String>,
    resolver: &Resolver,
) -> AppResult<ImportResult> {
    let selected_aliases: HashSet<String> = aliases.into_iter().collect();
    let candidates = import_preview_from_path(db, path, resolver)?;
    let mut imported = Vec::new();
    let mut skipped = 0usize;

    for candidate in candidates {
        if !selected_aliases.contains(&candidate.alias) {
            continue;
        }

        if candidate.skipped || candidate.duplicate {
            skipped += 1;
            continue;
        }

        let identity_file_id = match candidate.identity_file.as_deref() {
            Some(path) => Some(db.find_or_create_ssh_key_ref_for_path(path)?.id),
            None => None,
        };
        let server = db.create_server(ServerInput {
            display_name: candidate.name.clone(),
            host: candidate.host.clone(),
            port: u32::from(candidate.port),
            username: candidate.username.clone(),
            identity_file_id,
            group_id: None,
            notes: None,
            favorite: false,
            tag_names: Vec::new(),
        })?;
        imported.push(server);
    }

    Ok(ImportResult {
        imported: imported.len(),
        skipped,
        servers: imported,
    })
}

fn read_and_parse(path: &Path) -> AppResult<Vec<ParsedHost>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read SSH config at {}: {error}", path.display()))?;
    Ok(parse_ssh_config(&contents))
}

fn build_candidates(
    parsed_hosts: Vec<ParsedHost>,
    existing_servers: &[ServerProfile],
    resolver: &Resolver,
) -> Vec<ImportCandidate> {
    parsed_hosts
        .into_iter()
        .map(|parsed| {
            let mut warnings = parsed.warnings.clone();
            let resolved = if parsed.skipped {
                None
            } else {
                match resolver(&parsed.alias) {
                    Ok(value) => value,
                    Err(error) => {
                        warnings.push(format!(
                            "OpenSSH resolution failed for '{}': {error}. Parsed config values are shown.",
                            parsed.alias
                        ));
                        None
                    }
                }
            };

            let host = resolved
                .as_ref()
                .and_then(|value| value.host.clone())
                .or_else(|| parsed.host.clone())
                .unwrap_or_else(|| parsed.alias.clone());
            let username = resolved
                .as_ref()
                .and_then(|value| value.username.clone())
                .or_else(|| parsed.username.clone())
                .unwrap_or_default();
            let port = resolved
                .as_ref()
                .and_then(|value| value.port)
                .or(parsed.port)
                .unwrap_or(22);
            let identity_file = parsed.identity_file.clone();
            let proxy_jump = resolved
                .as_ref()
                .and_then(|value| value.proxy_jump.clone())
                .or_else(|| parsed.proxy_jump.clone());

            if proxy_jump.is_some() {
                warnings.push("ProxyJump detected; it is shown for review but is not stored or used for SSH launch yet.".to_string());
            }

            let duplicate = !parsed.skipped && is_duplicate(&parsed.alias, &host, existing_servers);
            if duplicate {
                warnings.push("Duplicate detected against an existing saved server.".to_string());
            }

            ImportCandidate {
                alias: parsed.alias.clone(),
                name: parsed.alias,
                host,
                port,
                username,
                identity_file,
                proxy_jump,
                warnings,
                selected: !parsed.skipped && !duplicate,
                duplicate,
                skipped: parsed.skipped,
            }
        })
        .collect()
}

fn is_duplicate(alias: &str, host: &str, existing_servers: &[ServerProfile]) -> bool {
    existing_servers.iter().any(|server| {
        server.display_name.eq_ignore_ascii_case(alias)
            || server.display_name.eq_ignore_ascii_case(host)
            || server.host.eq_ignore_ascii_case(host)
            || server.host.eq_ignore_ascii_case(alias)
    })
}

fn parse_ssh_config(contents: &str) -> Vec<ParsedHost> {
    let mut hosts = Vec::new();
    let mut current: Option<HostBlock> = None;

    for line in contents.lines() {
        let tokens = tokenize_ssh_config_line(line);
        if tokens.is_empty() {
            continue;
        }

        let keyword = tokens[0].to_ascii_lowercase();
        match keyword.as_str() {
            "host" => {
                flush_block(&mut hosts, current.take());
                current = Some(HostBlock {
                    patterns: tokens[1..].to_vec(),
                    ..HostBlock::default()
                });
            }
            "match" => {
                flush_block(&mut hosts, current.take());
            }
            _ => {
                if let Some(block) = current.as_mut() {
                    apply_option(block, &keyword, &tokens[1..]);
                }
            }
        }
    }

    flush_block(&mut hosts, current);
    hosts
}

fn apply_option(block: &mut HostBlock, keyword: &str, values: &[String]) {
    let Some(value) = values.first().cloned() else {
        return;
    };

    match keyword {
        "hostname" => block.host = Some(value),
        "user" => block.username = Some(value),
        "port" => match value.parse::<u16>() {
            Ok(port) if port > 0 => block.port = Some(port),
            _ => block
                .warnings
                .push(format!("Ignored invalid Port value '{}'.", value)),
        },
        "identityfile" => {
            if block.identity_file.is_none() {
                block.identity_file = Some(value);
            } else {
                block
                    .warnings
                    .push("Multiple IdentityFile values detected; using the first one.".to_string());
            }
        }
        "proxyjump" => block.proxy_jump = Some(value),
        _ => {}
    }
}

fn flush_block(hosts: &mut Vec<ParsedHost>, block: Option<HostBlock>) {
    let Some(block) = block else {
        return;
    };

    if block.patterns.is_empty() {
        return;
    }

    let (concrete, advanced): (Vec<_>, Vec<_>) = block
        .patterns
        .iter()
        .cloned()
        .partition(|pattern| is_concrete_host_pattern(pattern));

    if concrete.is_empty() {
        hosts.push(ParsedHost {
            alias: block.patterns.join(" "),
            host: None,
            username: None,
            port: None,
            identity_file: None,
            proxy_jump: None,
            warnings: vec!["Skipped wildcard or advanced Host pattern.".to_string()],
            skipped: true,
        });
        return;
    }

    for alias in concrete {
        let mut warnings = block.warnings.clone();
        if !advanced.is_empty() {
            warnings.push(format!(
                "Host block also contains wildcard or advanced patterns skipped by the importer: {}.",
                advanced.join(", ")
            ));
        }

        hosts.push(ParsedHost {
            alias,
            host: block.host.clone(),
            username: block.username.clone(),
            port: block.port,
            identity_file: block.identity_file.clone(),
            proxy_jump: block.proxy_jump.clone(),
            warnings,
            skipped: false,
        });
    }
}

fn is_concrete_host_pattern(pattern: &str) -> bool {
    !pattern.is_empty()
        && !pattern.starts_with('!')
        && !pattern.contains('*')
        && !pattern.contains('?')
}

fn resolve_with_ssh_g(alias: &str) -> AppResult<Option<ResolvedHostConfig>> {
    let output = Command::new("ssh")
        .args(["-G", alias])
        .output()
        .map_err(|error| format!("failed to run ssh -G: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(Some(parse_ssh_g_output(&String::from_utf8_lossy(
        &output.stdout,
    ))))
}

fn parse_ssh_g_output(output: &str) -> ResolvedHostConfig {
    let mut resolved = ResolvedHostConfig::default();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.splitn(2, char::is_whitespace);
        let Some(keyword) = parts.next() else {
            continue;
        };
        let value = parts.next().unwrap_or("").trim();
        if value.is_empty() {
            continue;
        }

        match keyword.to_ascii_lowercase().as_str() {
            "hostname" => resolved.host = Some(value.to_string()),
            "user" => resolved.username = Some(value.to_string()),
            "port" => resolved.port = value.parse::<u16>().ok().filter(|port| *port > 0),
            "identityfile" if resolved.identity_file.is_none() && value != "none" => {
                resolved.identity_file = Some(value.to_string())
            }
            "proxyjump" if value != "none" => resolved.proxy_jump = Some(value.to_string()),
            _ => {}
        }
    }
    resolved
}

fn tokenize_ssh_config_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for character in line.chars() {
        if escaped {
            token.push(character);
            escaped = false;
            continue;
        }

        if character == '\\' {
            escaped = true;
            continue;
        }

        if quote.is_none() && character == '#' {
            break;
        }

        if character == '"' || character == '\'' {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            } else {
                token.push(character);
            }
            continue;
        }

        if quote.is_none() && character.is_whitespace() {
            if !token.is_empty() {
                tokens.push(std::mem::take(&mut token));
            }
            continue;
        }

        token.push(character);
    }

    if !token.is_empty() {
        tokens.push(token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SshKeyInput;
    use tempfile::tempdir;

    fn test_db() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.migrate().unwrap();
        db
    }

    fn write_config(contents: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config");
        fs::write(&path, contents).unwrap();
        (dir, path)
    }

    fn no_resolver(_: &str) -> AppResult<Option<ResolvedHostConfig>> {
        Ok(None)
    }

    #[test]
    fn parses_simple_host_entries() {
        let parsed = parse_ssh_config(
            "
            Host nas
              HostName nas.local
              User admin
              Port 2222
            ",
        );

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].alias, "nas");
        assert_eq!(parsed[0].host.as_deref(), Some("nas.local"));
        assert_eq!(parsed[0].username.as_deref(), Some("admin"));
        assert_eq!(parsed[0].port, Some(2222));
    }

    #[test]
    fn skips_wildcard_host_entries() {
        let parsed = parse_ssh_config(
            "
            Host *
              User default
            Host router *.lab
              HostName router.local
            ",
        );

        assert_eq!(parsed.len(), 2);
        assert!(parsed[0].skipped);
        assert_eq!(parsed[1].alias, "router");
        assert!(parsed[1].warnings[0].contains("wildcard"));
    }

    #[test]
    fn handles_identity_file_and_proxy_jump() {
        let parsed = parse_ssh_config(
            "
            Host pve
              HostName 10.0.0.5
              User root
              IdentityFile ~/.ssh/pve
              ProxyJump bastion
            ",
        );

        assert_eq!(parsed[0].identity_file.as_deref(), Some("~/.ssh/pve"));
        assert_eq!(parsed[0].proxy_jump.as_deref(), Some("bastion"));
    }

    #[test]
    fn detects_duplicates_by_alias_and_hostname() {
        let db = test_db();
        db.create_server(ServerInput {
            display_name: "NAS".to_string(),
            host: "nas.local".to_string(),
            port: 22,
            username: "admin".to_string(),
            identity_file_id: None,
            group_id: None,
            notes: None,
            favorite: false,
            tag_names: Vec::new(),
        })
        .unwrap();
        let (_dir, path) = write_config(
            "
            Host nas
              HostName other.local
            Host router
              HostName nas.local
            ",
        );

        let preview = import_preview_from_path(&db, &path, &no_resolver).unwrap();

        assert_eq!(preview.len(), 2);
        assert!(preview[0].duplicate);
        assert!(preview[1].duplicate);
        assert!(!preview[0].selected);
        assert!(!preview[1].selected);
    }

    #[test]
    fn creates_identity_key_reference_during_import() {
        let db = test_db();
        let (_dir, path) = write_config(
            "
            Host nas
              HostName nas.local
              User admin
              IdentityFile ~/.ssh/id_nas
            ",
        );

        let result = import_selected_from_path(&db, &path, vec!["nas".to_string()], &no_resolver)
            .unwrap();

        assert_eq!(result.imported, 1);
        let keys = db.list_ssh_key_refs().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].path, "~/.ssh/id_nas");
        assert_eq!(
            db.get_server(&result.servers[0].id)
                .unwrap()
                .unwrap()
                .identity_file_id,
            Some(keys[0].id.clone())
        );
    }

    #[test]
    fn reuses_existing_identity_key_reference() {
        let db = test_db();
        let existing = db
            .create_ssh_key_ref(SshKeyInput {
                label: "Existing".to_string(),
                path: "~/.ssh/id_nas".to_string(),
                fingerprint: None,
                comment: None,
            })
            .unwrap();
        let (_dir, path) = write_config(
            "
            Host nas
              HostName nas.local
              IdentityFile ~/.ssh/id_nas
            ",
        );

        let result = import_selected_from_path(&db, &path, vec!["nas".to_string()], &no_resolver)
            .unwrap();

        assert_eq!(db.list_ssh_key_refs().unwrap().len(), 1);
        assert_eq!(result.servers[0].identity_file_id, Some(existing.id));
    }

    #[test]
    fn imports_selected_candidates_only() {
        let db = test_db();
        let (_dir, path) = write_config(
            "
            Host nas
              HostName nas.local
            Host router
              HostName router.local
            ",
        );

        let result = import_selected_from_path(&db, &path, vec!["router".to_string()], &no_resolver)
            .unwrap();

        assert_eq!(result.imported, 1);
        assert_eq!(result.servers[0].display_name, "router");
        assert_eq!(db.list_servers().unwrap().len(), 1);
    }

    #[test]
    fn missing_config_file_returns_empty_preview() {
        let db = test_db();
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing-config");

        assert!(import_preview_from_path(&db, &path, &no_resolver)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn parses_ssh_g_output() {
        let resolved = parse_ssh_g_output(
            "
            user root
            hostname 10.0.0.5
            port 2222
            identityfile ~/.ssh/id_pve
            proxyjump bastion
            ",
        );

        assert_eq!(resolved.username.as_deref(), Some("root"));
        assert_eq!(resolved.host.as_deref(), Some("10.0.0.5"));
        assert_eq!(resolved.port, Some(2222));
        assert_eq!(resolved.identity_file.as_deref(), Some("~/.ssh/id_pve"));
        assert_eq!(resolved.proxy_jump.as_deref(), Some("bastion"));
    }
}
