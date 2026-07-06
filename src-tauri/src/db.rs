use std::{
    path::Path,
    sync::{Mutex, MutexGuard},
    time::Duration,
};

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::domain::{
    default_settings, new_id, normalize_notes, normalize_optional, normalize_tag_names,
    normalize_text, now_timestamp, validate_app_settings, validate_group_input,
    validate_server_input, validate_ssh_key_input, validate_web_link_input, AppResult, AppSettings,
    Group, GroupInput, ServerInput, ServerProfile, SshKeyInput, SshKeyRef, Tag, WebLink,
    WebLinkInput,
};

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial", include_str!("../migrations/001_initial.sql")),
    (
        "002_app_settings",
        include_str!("../migrations/002_app_settings.sql"),
    ),
    (
        "003_server_web_links",
        include_str!("../migrations/003_server_web_links.sql"),
    ),
    (
        "004_server_proxy_jump",
        include_str!("../migrations/004_server_proxy_jump.sql"),
    ),
];

#[derive(Debug)]
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> AppResult<Self> {
        let conn = Connection::open(path).map_err(to_error)?;
        prepare_connection(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory().map_err(to_error)?;
        prepare_connection(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn migrate(&self) -> AppResult<()> {
        let mut conn = self.lock()?;
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS schema_migrations (
              version TEXT PRIMARY KEY,
              applied_at TEXT NOT NULL
            );
            ",
        )
        .map_err(to_error)?;

        for (version, sql) in MIGRATIONS {
            let applied = migration_applied(&conn, version)?;
            if applied {
                continue;
            }

            let tx = conn.transaction().map_err(to_error)?;
            tx.execute_batch(sql).map_err(to_error)?;
            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                params![version, now_timestamp()],
            )
            .map_err(to_error)?;
            tx.commit().map_err(to_error)?;
        }

        Ok(())
    }

    pub fn list_servers(&self) -> AppResult<Vec<ServerProfile>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                "
                SELECT id, display_name, host, port, username, identity_file_id, proxy_jump, group_id,
                       notes, favorite, created_at, updated_at
                FROM server_profiles
                ORDER BY favorite DESC, display_name COLLATE NOCASE ASC
                ",
            )
            .map_err(to_error)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ServerProfile {
                    id: row.get(0)?,
                    display_name: row.get(1)?,
                    host: row.get(2)?,
                    port: row.get::<_, i64>(3)? as u16,
                    username: row.get(4)?,
                    identity_file_id: row.get(5)?,
                    proxy_jump: row.get(6)?,
                    group_id: row.get(7)?,
                    notes: row.get(8)?,
                    favorite: row.get::<_, i64>(9)? == 1,
                    tags: Vec::new(),
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })
            .map_err(to_error)?;

        let mut servers = Vec::new();
        for row in rows {
            let mut server = row.map_err(to_error)?;
            server.tags = list_tags_for_server(&conn, &server.id)?;
            servers.push(server);
        }

        Ok(servers)
    }

    pub fn create_server(&self, input: ServerInput) -> AppResult<ServerProfile> {
        validate_server_input(&input)?;

        let id = new_id();
        let now = now_timestamp();
        {
            let mut conn = self.lock()?;
            let tx = conn.transaction().map_err(to_error)?;
            validate_references(
                &tx,
                input.group_id.as_deref(),
                input.identity_file_id.as_deref(),
            )?;

            tx.execute(
                "
                INSERT INTO server_profiles (
                  id, display_name, host, port, username, identity_file_id, proxy_jump, group_id,
                  notes, favorite, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ",
                params![
                    id,
                    normalize_text(&input.display_name).unwrap(),
                    normalize_text(&input.host).unwrap(),
                    input.port as i64,
                    input.username.trim(),
                    normalize_optional(input.identity_file_id),
                    normalize_optional(input.proxy_jump),
                    normalize_optional(input.group_id),
                    normalize_notes(input.notes),
                    bool_to_i64(input.favorite),
                    now,
                    now,
                ],
            )
            .map_err(to_error)?;
            sync_server_tags(&tx, &id, &input.tag_names)?;
            tx.commit().map_err(to_error)?;
        }

        self.get_server(&id)?
            .ok_or_else(|| "Created server was not found".to_string())
    }

    pub fn update_server(&self, id: &str, input: ServerInput) -> AppResult<ServerProfile> {
        validate_server_input(&input)?;

        {
            let mut conn = self.lock()?;
            let tx = conn.transaction().map_err(to_error)?;
            validate_references(
                &tx,
                input.group_id.as_deref(),
                input.identity_file_id.as_deref(),
            )?;

            let updated = tx
                .execute(
                    "
                    UPDATE server_profiles
                    SET display_name = ?1,
                        host = ?2,
                        port = ?3,
                        username = ?4,
                        identity_file_id = ?5,
                        proxy_jump = ?6,
                        group_id = ?7,
                        notes = ?8,
                        favorite = ?9,
                        updated_at = ?10
                    WHERE id = ?11
                    ",
                    params![
                        normalize_text(&input.display_name).unwrap(),
                        normalize_text(&input.host).unwrap(),
                        input.port as i64,
                        input.username.trim(),
                        normalize_optional(input.identity_file_id),
                        normalize_optional(input.proxy_jump),
                        normalize_optional(input.group_id),
                        normalize_notes(input.notes),
                        bool_to_i64(input.favorite),
                        now_timestamp(),
                        id,
                    ],
                )
                .map_err(to_error)?;

            if updated == 0 {
                return Err("Server not found".to_string());
            }

            sync_server_tags(&tx, id, &input.tag_names)?;
            tx.commit().map_err(to_error)?;
        }

        self.get_server(id)?
            .ok_or_else(|| "Updated server was not found".to_string())
    }

    pub fn delete_server(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM server_profiles WHERE id = ?1", params![id])
            .map_err(to_error)?;
        Ok(())
    }

    pub fn get_server(&self, id: &str) -> AppResult<Option<ServerProfile>> {
        let conn = self.lock()?;
        get_server_by_id(&conn, id)
    }

    pub fn list_groups(&self) -> AppResult<Vec<Group>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                "
                SELECT id, name, color, created_at, updated_at
                FROM groups
                ORDER BY name COLLATE NOCASE ASC
                ",
            )
            .map_err(to_error)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Group {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })
            .map_err(to_error)?;

        collect_rows(rows)
    }

    pub fn create_group(&self, input: GroupInput) -> AppResult<Group> {
        validate_group_input(&input)?;

        let conn = self.lock()?;
        let id = new_id();
        let now = now_timestamp();
        let group = Group {
            id,
            name: normalize_text(&input.name).unwrap(),
            color: normalize_optional(input.color),
            created_at: now.clone(),
            updated_at: now,
        };

        conn.execute(
            "
            INSERT INTO groups (id, name, color, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                group.id,
                group.name,
                group.color,
                group.created_at,
                group.updated_at
            ],
        )
        .map_err(to_error)?;

        Ok(group)
    }

    pub fn delete_group(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM groups WHERE id = ?1", params![id])
            .map_err(to_error)?;
        Ok(())
    }

    pub fn list_ssh_key_refs(&self) -> AppResult<Vec<SshKeyRef>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                "
                SELECT id, label, path, fingerprint, comment, created_at, updated_at
                FROM ssh_key_refs
                ORDER BY label COLLATE NOCASE ASC
                ",
            )
            .map_err(to_error)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(SshKeyRef {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    path: row.get(2)?,
                    fingerprint: row.get(3)?,
                    comment: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(to_error)?;

        collect_rows(rows)
    }

    pub fn create_ssh_key_ref(&self, input: SshKeyInput) -> AppResult<SshKeyRef> {
        validate_ssh_key_input(&input)?;

        let conn = self.lock()?;
        let id = new_id();
        let now = now_timestamp();
        let key = SshKeyRef {
            id,
            label: normalize_text(&input.label).unwrap(),
            path: normalize_text(&input.path).unwrap(),
            fingerprint: normalize_optional(input.fingerprint),
            comment: normalize_optional(input.comment),
            created_at: now.clone(),
            updated_at: now,
        };

        conn.execute(
            "
            INSERT INTO ssh_key_refs (id, label, path, fingerprint, comment, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                key.id,
                key.label,
                key.path,
                key.fingerprint,
                key.comment,
                key.created_at,
                key.updated_at
            ],
        )
        .map_err(to_error)?;

        Ok(key)
    }

    pub fn find_or_create_ssh_key_ref_for_path(&self, path: &str) -> AppResult<SshKeyRef> {
        let path = normalize_text(path).ok_or_else(|| "Key path is required".to_string())?;
        let conn = self.lock()?;

        if let Some(key) = conn
            .query_row(
                "
                SELECT id, label, path, fingerprint, comment, created_at, updated_at
                FROM ssh_key_refs
                WHERE path = ?1
                ",
                params![path],
                |row| {
                    Ok(SshKeyRef {
                        id: row.get(0)?,
                        label: row.get(1)?,
                        path: row.get(2)?,
                        fingerprint: row.get(3)?,
                        comment: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(to_error)?
        {
            return Ok(key);
        }

        let id = new_id();
        let now = now_timestamp();
        let key = SshKeyRef {
            id,
            label: key_label_from_path(&path),
            path,
            fingerprint: None,
            comment: Some("Imported from ~/.ssh/config".to_string()),
            created_at: now.clone(),
            updated_at: now,
        };

        conn.execute(
            "
            INSERT INTO ssh_key_refs (id, label, path, fingerprint, comment, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                key.id,
                key.label,
                key.path,
                key.fingerprint,
                key.comment,
                key.created_at,
                key.updated_at
            ],
        )
        .map_err(to_error)?;

        Ok(key)
    }

    pub fn delete_ssh_key_ref(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM ssh_key_refs WHERE id = ?1", params![id])
            .map_err(to_error)?;
        Ok(())
    }

    pub fn list_tags(&self) -> AppResult<Vec<Tag>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT id, name, created_at, updated_at FROM tags ORDER BY name COLLATE NOCASE ASC")
            .map_err(to_error)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })
            .map_err(to_error)?;

        collect_rows(rows)
    }

    pub fn get_settings(&self) -> AppResult<AppSettings> {
        let conn = self.lock()?;
        let settings = conn
            .query_row(
                "
                SELECT terminal_preference, safety_warnings_enabled
                FROM app_settings
                WHERE id = 1
                ",
                [],
                |row| {
                    Ok(AppSettings {
                        terminal_preference: row.get(0)?,
                        safety_warnings_enabled: row.get::<_, i64>(1)? == 1,
                    })
                },
            )
            .optional()
            .map_err(to_error)?;

        Ok(settings.unwrap_or_else(default_settings))
    }

    pub fn save_settings(&self, input: AppSettings) -> AppResult<AppSettings> {
        validate_app_settings(&input)?;

        let conn = self.lock()?;
        let updated = conn
            .execute(
                "
                UPDATE app_settings
                SET terminal_preference = ?1,
                    safety_warnings_enabled = ?2
                WHERE id = 1
                ",
                params![
                    &input.terminal_preference,
                    bool_to_i64(input.safety_warnings_enabled)
                ],
            )
            .map_err(to_error)?;

        if updated == 0 {
            conn.execute(
                "
                INSERT INTO app_settings (id, terminal_preference, safety_warnings_enabled)
                VALUES (1, ?1, ?2)
                ",
                params![
                    &input.terminal_preference,
                    bool_to_i64(input.safety_warnings_enabled)
                ],
            )
            .map_err(to_error)?;
        }

        drop(conn);
        self.get_settings()
    }

    pub fn key_path(&self, id: &str) -> AppResult<Option<String>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT path FROM ssh_key_refs WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()
        .map_err(to_error)
    }

    pub fn list_web_links(&self, server_id: &str) -> AppResult<Vec<WebLink>> {
        let conn = self.lock()?;
        ensure_server_exists(&conn, server_id)?;
        list_web_links_for_server(&conn, server_id)
    }

    pub fn create_web_link(&self, server_id: &str, input: WebLinkInput) -> AppResult<WebLink> {
        validate_web_link_input(&input)?;

        let conn = self.lock()?;
        ensure_server_exists(&conn, server_id)?;
        let id = new_id();
        let now = now_timestamp();
        let link = WebLink {
            id,
            server_profile_id: server_id.to_string(),
            label: normalize_text(&input.label).unwrap(),
            url: input.url.trim().to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        conn.execute(
            "
            INSERT INTO server_web_links (
              id, server_profile_id, label, url, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                link.id,
                link.server_profile_id,
                link.label,
                link.url,
                link.created_at,
                link.updated_at
            ],
        )
        .map_err(to_error)?;

        Ok(link)
    }

    pub fn update_web_link(&self, id: &str, input: WebLinkInput) -> AppResult<WebLink> {
        validate_web_link_input(&input)?;

        let conn = self.lock()?;
        let updated = conn
            .execute(
                "
                UPDATE server_web_links
                SET label = ?1,
                    url = ?2,
                    updated_at = ?3
                WHERE id = ?4
                ",
                params![
                    normalize_text(&input.label).unwrap(),
                    input.url.trim(),
                    now_timestamp(),
                    id
                ],
            )
            .map_err(to_error)?;

        if updated == 0 {
            return Err("Web link not found".to_string());
        }

        get_web_link_by_id(&conn, id)?.ok_or_else(|| "Updated web link was not found".to_string())
    }

    pub fn delete_web_link(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        let deleted = conn
            .execute("DELETE FROM server_web_links WHERE id = ?1", params![id])
            .map_err(to_error)?;

        if deleted == 0 {
            return Err("Web link not found".to_string());
        }

        Ok(())
    }

    pub fn get_web_link_for_server(&self, server_id: &str, link_id: &str) -> AppResult<WebLink> {
        let conn = self.lock()?;
        ensure_server_exists(&conn, server_id)?;
        let link =
            get_web_link_by_id(&conn, link_id)?.ok_or_else(|| "Web link not found".to_string())?;

        if link.server_profile_id != server_id {
            return Err("Web link does not belong to this server".to_string());
        }

        Ok(link)
    }

    #[cfg(test)]
    pub fn table_exists(&self, table_name: &str) -> AppResult<bool> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            params![table_name],
            |row| row.get::<_, i64>(0),
        )
        .map(|value| value == 1)
        .map_err(to_error)
    }

    #[cfg(test)]
    pub fn column_exists(&self, table_name: &str, column_name: &str) -> AppResult<bool> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table_name})"))
            .map_err(to_error)?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(to_error)?;

        for row in rows {
            if row.map_err(to_error)? == column_name {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn lock(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| "Database lock was poisoned".to_string())
    }
}

fn prepare_connection(conn: &Connection) -> AppResult<()> {
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(to_error)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(to_error)?;
    Ok(())
}

fn migration_applied(conn: &Connection, version: &str) -> AppResult<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
        params![version],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value == 1)
    .map_err(to_error)
}

fn validate_references(
    conn: &Connection,
    group_id: Option<&str>,
    identity_file_id: Option<&str>,
) -> AppResult<()> {
    if let Some(id) = group_id.and_then(normalize_text) {
        let exists = exists_by_id(conn, "groups", &id)?;
        if !exists {
            return Err("Selected group does not exist".to_string());
        }
    }

    if let Some(id) = identity_file_id.and_then(normalize_text) {
        let exists = exists_by_id(conn, "ssh_key_refs", &id)?;
        if !exists {
            return Err("Selected SSH key reference does not exist".to_string());
        }
    }

    Ok(())
}

fn exists_by_id(conn: &Connection, table: &str, id: &str) -> AppResult<bool> {
    let sql = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = ?1)");
    conn.query_row(&sql, params![id], |row| row.get::<_, i64>(0))
        .map(|value| value == 1)
        .map_err(to_error)
}

fn ensure_server_exists(conn: &Connection, server_id: &str) -> AppResult<()> {
    if exists_by_id(conn, "server_profiles", server_id)? {
        Ok(())
    } else {
        Err("Server not found".to_string())
    }
}

fn sync_server_tags(tx: &Transaction<'_>, server_id: &str, tag_names: &[String]) -> AppResult<()> {
    tx.execute(
        "DELETE FROM server_profile_tags WHERE server_profile_id = ?1",
        params![server_id],
    )
    .map_err(to_error)?;

    for tag_name in normalize_tag_names(tag_names) {
        let tag = find_or_create_tag(tx, &tag_name)?;
        tx.execute(
            "
            INSERT OR IGNORE INTO server_profile_tags (server_profile_id, tag_id)
            VALUES (?1, ?2)
            ",
            params![server_id, tag.id],
        )
        .map_err(to_error)?;
    }

    Ok(())
}

fn find_or_create_tag(conn: &Connection, name: &str) -> AppResult<Tag> {
    if let Some(tag) = conn
        .query_row(
            "SELECT id, name, created_at, updated_at FROM tags WHERE name = ?1 COLLATE NOCASE",
            params![name],
            |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(to_error)?
    {
        return Ok(tag);
    }

    let now = now_timestamp();
    let tag = Tag {
        id: new_id(),
        name: name.to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO tags (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![tag.id, tag.name, tag.created_at, tag.updated_at],
    )
    .map_err(to_error)?;
    Ok(tag)
}

fn get_server_by_id(conn: &Connection, id: &str) -> AppResult<Option<ServerProfile>> {
    let mut server = conn
        .query_row(
            "
            SELECT id, display_name, host, port, username, identity_file_id, proxy_jump, group_id,
                   notes, favorite, created_at, updated_at
            FROM server_profiles
            WHERE id = ?1
            ",
            params![id],
            |row| {
                Ok(ServerProfile {
                    id: row.get(0)?,
                    display_name: row.get(1)?,
                    host: row.get(2)?,
                    port: row.get::<_, i64>(3)? as u16,
                    username: row.get(4)?,
                    identity_file_id: row.get(5)?,
                    proxy_jump: row.get(6)?,
                    group_id: row.get(7)?,
                    notes: row.get(8)?,
                    favorite: row.get::<_, i64>(9)? == 1,
                    tags: Vec::new(),
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        )
        .optional()
        .map_err(to_error)?;

    if let Some(server) = &mut server {
        server.tags = list_tags_for_server(conn, &server.id)?;
    }

    Ok(server)
}

fn list_tags_for_server(conn: &Connection, server_id: &str) -> AppResult<Vec<Tag>> {
    let mut stmt = conn
        .prepare(
            "
            SELECT t.id, t.name, t.created_at, t.updated_at
            FROM tags t
            INNER JOIN server_profile_tags spt ON spt.tag_id = t.id
            WHERE spt.server_profile_id = ?1
            ORDER BY t.name COLLATE NOCASE ASC
            ",
        )
        .map_err(to_error)?;
    let rows = stmt
        .query_map(params![server_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(to_error)?;

    collect_rows(rows)
}

fn list_web_links_for_server(conn: &Connection, server_id: &str) -> AppResult<Vec<WebLink>> {
    let mut stmt = conn
        .prepare(
            "
            SELECT id, server_profile_id, label, url, created_at, updated_at
            FROM server_web_links
            WHERE server_profile_id = ?1
            ORDER BY label COLLATE NOCASE ASC
            ",
        )
        .map_err(to_error)?;
    let rows = stmt
        .query_map(params![server_id], web_link_from_row)
        .map_err(to_error)?;

    collect_rows(rows)
}

fn get_web_link_by_id(conn: &Connection, id: &str) -> AppResult<Option<WebLink>> {
    conn.query_row(
        "
        SELECT id, server_profile_id, label, url, created_at, updated_at
        FROM server_web_links
        WHERE id = ?1
        ",
        params![id],
        web_link_from_row,
    )
    .optional()
    .map_err(to_error)
}

fn web_link_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WebLink> {
    Ok(WebLink {
        id: row.get(0)?,
        server_profile_id: row.get(1)?,
        label: row.get(2)?,
        url: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> AppResult<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row.map_err(to_error)?);
    }
    Ok(values)
}

fn bool_to_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn key_label_from_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    let label = trimmed.rsplit('/').next().unwrap_or(trimmed).trim();
    if label.is_empty() {
        path.to_string()
    } else {
        label.to_string()
    }
}

fn to_error(error: rusqlite::Error) -> String {
    format!("Database error: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AppSettings, GroupInput, ServerInput, SshKeyInput, WebLinkInput};

    fn test_db() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.migrate().unwrap();
        db
    }

    fn server_input() -> ServerInput {
        ServerInput {
            display_name: "NAS".to_string(),
            host: "nas.local".to_string(),
            port: 22,
            username: "admin".to_string(),
            identity_file_id: None,
            proxy_jump: Some("bastion".to_string()),
            group_id: None,
            notes: Some("local metadata".to_string()),
            favorite: false,
            tag_names: vec!["linux".to_string(), "storage".to_string()],
        }
    }

    #[test]
    fn migrations_create_expected_tables_and_are_idempotent() {
        let db = test_db();
        db.migrate().unwrap();

        for table in [
            "schema_migrations",
            "groups",
            "ssh_key_refs",
            "server_profiles",
            "tags",
            "server_profile_tags",
            "app_settings",
            "server_web_links",
        ] {
            assert!(db.table_exists(table).unwrap(), "{table} should exist");
        }
        assert!(db.column_exists("server_profiles", "proxy_jump").unwrap());
    }

    #[test]
    fn settings_round_trip_and_reject_invalid_terminal() {
        let db = test_db();
        assert_eq!(db.get_settings().unwrap().terminal_preference, "auto");

        let saved = db
            .save_settings(AppSettings {
                terminal_preference: "konsole".to_string(),
                safety_warnings_enabled: false,
            })
            .unwrap();
        assert_eq!(saved.terminal_preference, "konsole");
        assert!(!saved.safety_warnings_enabled);

        let loaded = db.get_settings().unwrap();
        assert_eq!(loaded, saved);

        assert_eq!(
            db.save_settings(AppSettings {
                terminal_preference: "powershell".to_string(),
                safety_warnings_enabled: true,
            })
            .unwrap_err(),
            "Unsupported terminal preference"
        );
    }

    #[test]
    fn server_crud_round_trip() {
        let db = test_db();
        let created = db.create_server(server_input()).unwrap();
        assert_eq!(created.display_name, "NAS");
        assert_eq!(created.proxy_jump.as_deref(), Some("bastion"));
        assert_eq!(created.tags.len(), 2);

        let mut update = server_input();
        update.display_name = "NAS Updated".to_string();
        update.proxy_jump = Some("admin@jump.local:2222".to_string());
        update.favorite = true;
        update.tag_names = vec!["prod".to_string()];
        let updated = db.update_server(&created.id, update).unwrap();
        assert_eq!(updated.display_name, "NAS Updated");
        assert_eq!(updated.proxy_jump.as_deref(), Some("admin@jump.local:2222"));
        assert!(updated.favorite);
        assert_eq!(updated.tags[0].name, "prod");

        assert_eq!(db.list_servers().unwrap().len(), 1);
        db.delete_server(&created.id).unwrap();
        assert!(db.list_servers().unwrap().is_empty());
    }

    #[test]
    fn web_link_crud_round_trip() {
        let db = test_db();
        let server = db.create_server(server_input()).unwrap();

        let created = db
            .create_web_link(
                &server.id,
                WebLinkInput {
                    label: "Proxmox".to_string(),
                    url: "https://pve.local:8006".to_string(),
                },
            )
            .unwrap();
        assert_eq!(created.label, "Proxmox");
        assert_eq!(created.server_profile_id, server.id);

        let links = db.list_web_links(&server.id).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://pve.local:8006");

        let updated = db
            .update_web_link(
                &created.id,
                WebLinkInput {
                    label: "Router".to_string(),
                    url: "http://router.local".to_string(),
                },
            )
            .unwrap();
        assert_eq!(updated.label, "Router");
        assert_eq!(updated.url, "http://router.local");

        db.delete_web_link(&created.id).unwrap();
        assert!(db.list_web_links(&server.id).unwrap().is_empty());
    }

    #[test]
    fn rejects_invalid_web_link_urls() {
        let db = test_db();
        let server = db.create_server(server_input()).unwrap();

        for url in [
            "",
            "not a url",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/plain,hello",
            "https://user:secret@example.com",
            "https://",
        ] {
            let error = db
                .create_web_link(
                    &server.id,
                    WebLinkInput {
                        label: "Bad".to_string(),
                        url: url.to_string(),
                    },
                )
                .unwrap_err();
            assert!(
                error.contains("Web link URL") || error.contains("embedded credentials"),
                "unexpected error for {url:?}: {error}"
            );
        }
    }

    #[test]
    fn deleting_server_cascades_web_links() {
        let db = test_db();
        let server = db.create_server(server_input()).unwrap();
        let link = db
            .create_web_link(
                &server.id,
                WebLinkInput {
                    label: "Admin".to_string(),
                    url: "https://admin.local".to_string(),
                },
            )
            .unwrap();

        db.delete_server(&server.id).unwrap();
        let conn = db.lock().unwrap();
        let count = conn
            .query_row(
                "SELECT COUNT(*) FROM server_web_links WHERE id = ?1",
                params![link.id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn web_link_lookup_rejects_server_mismatch() {
        let db = test_db();
        let first = db.create_server(server_input()).unwrap();
        let mut second_input = server_input();
        second_input.display_name = "Router".to_string();
        second_input.host = "router.local".to_string();
        let second = db.create_server(second_input).unwrap();
        let link = db
            .create_web_link(
                &first.id,
                WebLinkInput {
                    label: "Admin".to_string(),
                    url: "https://admin.local".to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            db.get_web_link_for_server(&second.id, &link.id)
                .unwrap_err(),
            "Web link does not belong to this server"
        );
    }

    #[test]
    fn foreign_keys_set_nullable_references_to_null() {
        let db = test_db();
        let group = db
            .create_group(GroupInput {
                name: "Lab".to_string(),
                color: Some("#3aa675".to_string()),
            })
            .unwrap();
        let key = db
            .create_ssh_key_ref(SshKeyInput {
                label: "Default".to_string(),
                path: "~/.ssh/id_ed25519".to_string(),
                fingerprint: None,
                comment: None,
            })
            .unwrap();

        let mut input = server_input();
        input.group_id = Some(group.id.clone());
        input.identity_file_id = Some(key.id.clone());
        let server = db.create_server(input).unwrap();

        db.delete_group(&group.id).unwrap();
        db.delete_ssh_key_ref(&key.id).unwrap();

        let server = db.get_server(&server.id).unwrap().unwrap();
        assert!(server.group_id.is_none());
        assert!(server.identity_file_id.is_none());
    }

    #[test]
    fn rejects_missing_references() {
        let db = test_db();
        let mut input = server_input();
        input.group_id = Some("missing".to_string());
        assert_eq!(
            db.create_server(input).unwrap_err(),
            "Selected group does not exist"
        );

        let mut input = server_input();
        input.identity_file_id = Some("missing".to_string());
        assert_eq!(
            db.create_server(input).unwrap_err(),
            "Selected SSH key reference does not exist"
        );
    }
}
