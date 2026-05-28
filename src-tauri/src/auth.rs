use bcrypt::{hash, verify, DEFAULT_COST};
use duckdb::params;
use serde::Serialize;
use tauri::State;

use crate::db::DbState;

#[derive(Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub nickname: String,
}

pub(crate) struct UserRow {
    pub id: i64,
    pub name: String,
    pub nickname: String,
    pub password_hash: String,
    pub active: bool,
}

pub fn lookup_user(conn: &duckdb::Connection, identifier: &str) -> Result<Option<UserRow>, String> {
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
pub fn check_setup(state: State<DbState>) -> Result<bool, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    crate::db::init_schema(&conn)?;
    crate::db::ensure_user_settings_table(&conn)?;
    crate::db::ensure_unit_column(&conn)?;
    crate::db::ensure_profit_margin_column(&conn)?;
    crate::db::ensure_label_column(&conn)?;
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
pub fn create_user(
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
pub fn check_business_setup(state: State<DbState>) -> Result<bool, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn
        .query_row(
            r#"SELECT COUNT(*) FROM "Business_Settings""#,
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(count > 0)
}

#[tauri::command]
pub fn create_business_settings(
    name: String,
    investment_rate: f64,
    profit_margin: f64,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();

    if name.is_empty() {
        return Err("Nome da empresa é obrigatório".to_string());
    }
    if !(0.0..=1.0).contains(&investment_rate) || !(0.0..=1.0).contains(&profit_margin) {
        return Err("Taxas devem estar entre 0% e 100%".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        r#"INSERT INTO "Business_Settings" (name, investment_rate, profit_margin, updated_at, created_at)
           VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name, investment_rate, profit_margin],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn find_user(identifier: String, state: State<DbState>) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    lookup_user(&conn, identifier.trim()).map(|u| u.map(|row| row.nickname))
}

#[tauri::command]
pub fn login(identifier: String, password: String, state: State<DbState>) -> Result<User, String> {
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

#[tauri::command]
pub fn get_user_setting(
    user_id: i64,
    key: String,
    state: State<DbState>,
) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(r#"SELECT value FROM "User_Settings" WHERE user_id = ? AND key = ?"#)
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map(params![user_id, key], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    match rows.next() {
        Some(Ok(value)) => Ok(Some(value)),
        Some(Err(e)) => Err(e.to_string()),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn set_user_setting(
    user_id: i64,
    key: String,
    value: String,
    state: State<DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        r#"INSERT INTO "User_Settings" (user_id, key, value)
           VALUES (?, ?, ?)
           ON CONFLICT (user_id, key) DO UPDATE SET value = EXCLUDED.value"#,
        params![user_id, key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn insert_user(conn: &duckdb::Connection, name: &str, nickname: &str, active: bool) {
    let pw_hash = hash("senha123", DEFAULT_COST).unwrap();
    conn.execute(
        r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
           VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name, nickname, pw_hash, active],
    )
    .unwrap();
}

#[cfg(test)]
fn find_user_in_test(conn: &duckdb::Connection, identifier: &str) -> Option<String> {
    lookup_user(conn, identifier).unwrap().map(|row| row.nickname)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_db;
    use crate::products::insert_business_settings;

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

    #[test]
    fn check_business_setup_empty_returns_false() {
        let conn = test_db();
        let count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Business_Settings""#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn check_business_setup_with_record_returns_true() {
        let conn = test_db();
        insert_business_settings(&conn, "Minha Empresa");
        let count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Business_Settings""#, [], |r| r.get(0))
            .unwrap();
        assert!(count > 0);
    }

    #[test]
    fn create_business_settings_rejects_empty_name() {
        let name = "  ".trim().to_string();
        assert!(name.is_empty());
    }

    #[test]
    fn create_business_settings_rejects_rate_out_of_range() {
        assert!(!(0.0_f64..=1.0).contains(&1.5));
        assert!(!(0.0_f64..=1.0).contains(&-0.1));
        assert!((0.0_f64..=1.0).contains(&0.5));
    }

    #[test]
    fn login_rejects_inactive_user() {
        let conn = test_db();
        insert_user(&conn, "Inativo", "inativo", false);
        let result = lookup_user(&conn, "inativo").unwrap();
        assert!(!result.unwrap().active);
    }

    #[test]
    fn get_user_setting_returns_none_when_absent() {
        let conn = test_db();
        insert_user(&conn, "Ana", "ana", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" WHERE nickname = 'ana'"#, [], |r| r.get(0))
            .unwrap();
        let mut stmt = conn
            .prepare(r#"SELECT value FROM "User_Settings" WHERE user_id = ? AND key = ?"#)
            .unwrap();
        let mut rows = stmt.query_map(params![user_id, "theme"], |r| r.get::<_, String>(0)).unwrap();
        assert!(rows.next().is_none());
    }

    #[test]
    fn set_user_setting_inserts_and_get_returns_value() {
        let conn = test_db();
        insert_user(&conn, "Ana", "ana", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" WHERE nickname = 'ana'"#, [], |r| r.get(0))
            .unwrap();
        conn.execute(
            r#"INSERT INTO "User_Settings" (user_id, key, value) VALUES (?, ?, ?)"#,
            params![user_id, "theme", "dark"],
        ).unwrap();
        let value: String = conn
            .query_row(
                r#"SELECT value FROM "User_Settings" WHERE user_id = ? AND key = ?"#,
                params![user_id, "theme"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(value, "dark");
    }

    #[test]
    fn set_user_setting_upserts_existing_value() {
        let conn = test_db();
        insert_user(&conn, "Ana", "ana", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" WHERE nickname = 'ana'"#, [], |r| r.get(0))
            .unwrap();
        conn.execute(
            r#"INSERT INTO "User_Settings" (user_id, key, value) VALUES (?, ?, ?)"#,
            params![user_id, "theme", "dark"],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "User_Settings" (user_id, key, value)
               VALUES (?, ?, ?)
               ON CONFLICT (user_id, key) DO UPDATE SET value = EXCLUDED.value"#,
            params![user_id, "theme", "light"],
        ).unwrap();
        let value: String = conn
            .query_row(
                r#"SELECT value FROM "User_Settings" WHERE user_id = ? AND key = ?"#,
                params![user_id, "theme"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(value, "light");
    }
}
