use bcrypt::{hash, verify, DEFAULT_COST};
use duckdb::{params, Connection};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, State};

const SCHEMA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/db/schema.sql"));

pub struct DbState(pub Mutex<Connection>);

#[derive(Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub nickname: String,
}

struct UserRow {
    id: i64,
    name: String,
    nickname: String,
    password_hash: String,
    active: bool,
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    // execute_batch aborts DuckDB when objects already exist, even with IF NOT EXISTS.
    // Skip entirely if the schema was already initialized in a previous run.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'Financial_Entries'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if count > 0 {
        return Ok(());
    }

    conn.execute_batch(SCHEMA).map_err(|e| e.to_string())
}

fn lookup_user(conn: &Connection, identifier: &str) -> Result<Option<UserRow>, String> {
    // Single query for both nickname and numeric-ID lookup. Avoids running a
    // query_row that returns QueryReturnedNoRows, which leaves DuckDB in a bad
    // connection state and causes a crash on the next operation.
    let mut stmt = conn
        .prepare(r#"
            SELECT id, name, nickname, password_hash, active
            FROM "Users"
            WHERE lower(nickname) = lower(?) OR id = TRY_CAST(? AS INTEGER)
            ORDER BY CASE WHEN lower(nickname) = lower(?) THEN 0 ELSE 1 END
            LIMIT 1
        "#)
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query_map(params![identifier, identifier, identifier], |row| {
            Ok(UserRow {
                id: row.get(0)?,
                name: row.get(1)?,
                nickname: row.get(2)?,
                password_hash: row.get(3)?,
                active: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    match rows.next() {
        Some(Ok(user)) => Ok(Some(user)),
        Some(Err(e)) => Err(e.to_string()),
        None => Ok(None),
    }
}

#[tauri::command]
fn check_setup(state: State<DbState>) -> Result<bool, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    init_schema(&conn)?;
    let count: i64 = conn
        .query_row(
            r#"SELECT COUNT(*) FROM "Users" WHERE active = true"#,
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

#[tauri::command]
fn create_user(
    name: String,
    nickname: String,
    password: String,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    let nickname = nickname.trim().to_string();
    let password = password.trim().to_string();

    if name.is_empty() || nickname.is_empty() || password.is_empty() {
        return Err("Todos os campos são obrigatórios".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let count: i64 = conn
        .query_row(
            r#"SELECT COUNT(*) FROM "Users" WHERE lower(nickname) = lower(?)"#,
            params![nickname],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count > 0 {
        return Err("Apelido já está em uso".to_string());
    }

    let password_hash = hash(&password, DEFAULT_COST).map_err(|e| e.to_string())?;

    conn.execute(
        r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
           VALUES (?, ?, ?, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name, nickname, password_hash],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn find_user(identifier: String, state: State<DbState>) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    lookup_user(&conn, identifier.trim()).map(|u| u.map(|row| row.nickname))
}

#[tauri::command]
fn login(identifier: String, password: String, state: State<DbState>) -> Result<User, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let row = lookup_user(&conn, identifier.trim())?
        .ok_or_else(|| "Credenciais inválidas".to_string())?;

    if !row.active {
        return Err("Conta inativa".to_string());
    }

    let valid = verify(password.trim(), &row.password_hash).map_err(|e| e.to_string())?;
    if !valid {
        return Err("Credenciais inválidas".to_string());
    }

    Ok(User { id: row.id, name: row.name, nickname: row.nickname })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = match std::env::var("DB_PATH") {
                Ok(p) => std::path::PathBuf::from(p),
                Err(_) => {
                    let data_dir = app.path().app_data_dir()?;
                    std::fs::create_dir_all(&data_dir)?;
                    data_dir.join("teller.db")
                }
            };
            let conn = Connection::open(db_path)?;
            app.manage(DbState(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_setup,
            create_user,
            find_user,
            login
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    fn insert_user(conn: &Connection, name: &str, nickname: &str, active: bool) {
        let pw_hash = hash("senha123", DEFAULT_COST).unwrap();
        conn.execute(
            r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
               VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![name, nickname, pw_hash, active],
        )
        .unwrap();
    }

    #[test]
    fn check_setup_empty_db_returns_false() {
        let conn = test_db();
        let count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Users""#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn check_setup_with_users_returns_true() {
        let conn = test_db();
        insert_user(&conn, "João Silva", "joao", true);
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Users" WHERE active = true"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(count > 0);
    }

    #[test]
    fn lookup_user_finds_by_nickname_case_insensitive() {
        let conn = test_db();
        insert_user(&conn, "João Silva", "João", true);
        let nickname = find_user_in_test(&conn, "joão");
        assert_eq!(nickname, Some("João".to_string()));
    }

    #[test]
    fn lookup_user_finds_by_numeric_id() {
        let conn = test_db();
        insert_user(&conn, "Alice Costa", "alice", true);
        let id: i64 = conn
            .query_row(
                r#"SELECT id FROM "Users" WHERE nickname = 'alice'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        let nickname = find_user_in_test(&conn, &id.to_string());
        assert_eq!(nickname, Some("alice".to_string()));
    }

    #[test]
    fn lookup_user_returns_none_for_unknown() {
        let conn = test_db();
        let nickname = find_user_in_test(&conn, "ninguem");
        assert_eq!(nickname, None);
    }

    fn find_user_in_test(conn: &Connection, identifier: &str) -> Option<String> {
        lookup_user(conn, identifier).unwrap().map(|row| row.nickname)
    }

    #[test]
    fn login_rejects_inactive_user() {
        let conn = test_db();
        insert_user(&conn, "Inativo", "inativo", false);
        let result = lookup_user(&conn, "inativo").unwrap();
        assert!(!result.unwrap().active);
    }
}
