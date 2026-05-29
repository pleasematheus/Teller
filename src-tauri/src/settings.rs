use bcrypt::{hash, DEFAULT_COST};
use duckdb::params;
use serde::Serialize;
use tauri::State;

use crate::db::DbState;

#[derive(Serialize)]
pub struct BusinessInfo {
    pub name: String,
    pub investment_rate: Option<f64>,
    pub profit_margin: Option<f64>,
}

#[derive(Serialize)]
pub struct PaymentMethod {
    pub id: i64,
    pub name: String,
    pub max_fee_percent: f64,
    pub active: bool,
}

#[derive(Serialize)]
pub struct TransactionFee {
    pub id: i64,
    pub payment_method_id: i64,
    pub description: String,
    pub fee_percent: f64,
    pub fixed_fee: f64,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub name: String,
    pub nickname: String,
    pub active: bool,
}

#[tauri::command]
pub fn list_payment_methods(state: State<DbState>) -> Result<Vec<PaymentMethod>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT pm.id, pm.name, COALESCE(MAX(tf.fee_percent), 0.0), pm.active
            FROM "Payment_Methods" pm
            LEFT JOIN "Transaction_Fees" tf ON tf.payment_method_id = pm.id
            GROUP BY pm.id, pm.name, pm.active
            ORDER BY pm.active DESC, pm.name
        "#)
        .map_err(|e| e.to_string())?;

    let methods = stmt
        .query_map([], |row| {
            Ok(PaymentMethod {
                id: row.get(0)?,
                name: row.get(1)?,
                max_fee_percent: row.get(2)?,
                active: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(methods)
}

#[tauri::command]
pub fn get_business_info(state: State<DbState>) -> Result<BusinessInfo, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let info = conn
        .query_row(
            r#"SELECT name, investment_rate, profit_margin FROM "Business_Settings" LIMIT 1"#,
            [],
            |row| {
                Ok(BusinessInfo {
                    name: row.get(0)?,
                    investment_rate: row.get(1)?,
                    profit_margin: row.get(2)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(info)
}

#[tauri::command]
pub fn update_business_settings(
    name: String,
    investment_rate: Option<f64>,
    profit_margin: Option<f64>,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();

    if name.is_empty() {
        return Err("Nome da empresa não pode ser vazio".to_string());
    }
    if investment_rate.map_or(false, |r| !(0.0..=1.0).contains(&r)) {
        return Err("Taxa de investimento deve estar entre 0% e 100%".to_string());
    }
    if profit_margin.map_or(false, |r| !(0.0..=1.0).contains(&r)) {
        return Err("Margem de lucro deve estar entre 0% e 100%".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    // Columns are DOUBLE (not DECIMAL): Option<f64> binds and reads correctly via params!.
    let ir: &dyn duckdb::types::ToSql = match &investment_rate { Some(v) => v, None => &duckdb::types::Null };
    let pm: &dyn duckdb::types::ToSql = match &profit_margin { Some(v) => v, None => &duckdb::types::Null };
    conn.execute(
        r#"UPDATE "Business_Settings"
           SET name = ?, investment_rate = ?, profit_margin = ?, updated_at = CURRENT_TIMESTAMP"#,
        &[&name as &dyn duckdb::types::ToSql, ir, pm],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn list_users(state: State<DbState>) -> Result<Vec<UserInfo>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"SELECT id, name, nickname, active FROM "Users" ORDER BY name"#)
        .map_err(|e| e.to_string())?;

    let users = stmt
        .query_map([], |row| {
            Ok(UserInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                nickname: row.get(2)?,
                active: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(users)
}

#[tauri::command]
pub fn change_user_password(
    user_id: i64,
    password: String,
    state: State<DbState>,
) -> Result<(), String> {
    let password = password.trim().to_string();
    if password.is_empty() {
        return Err("Senha não pode ser vazia".to_string());
    }

    let password_hash = hash(&password, DEFAULT_COST).map_err(|e| e.to_string())?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let rows = conn
        .execute(
            r#"UPDATE "Users" SET password_hash = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![password_hash, user_id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Usuário {} não encontrado", user_id));
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_user_active(user_id: i64, active: bool, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    if !active {
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Users" WHERE active = true AND id != ?"#,
                params![user_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if count == 0 {
            return Err("Não é possível desativar o último usuário ativo".to_string());
        }
    }

    let rows = conn
        .execute(
            r#"UPDATE "Users" SET active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![active, user_id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Usuário {} não encontrado", user_id));
    }
    Ok(())
}

#[tauri::command]
pub fn create_payment_method(name: String, state: State<DbState>) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Nome é obrigatório".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        r#"INSERT INTO "Payment_Methods" (name, active, updated_at, created_at)
           VALUES (?, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_payment_method(id: i64, active: bool, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let rows = conn
        .execute(
            r#"UPDATE "Payment_Methods" SET active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![active, id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Forma de pagamento {} não encontrada", id));
    }
    Ok(())
}

#[tauri::command]
pub fn list_transaction_fees(
    payment_method_id: i64,
    state: State<DbState>,
) -> Result<Vec<TransactionFee>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT id, payment_method_id, description, fee_percent, fixed_fee
            FROM "Transaction_Fees"
            WHERE payment_method_id = ?
            ORDER BY description
        "#)
        .map_err(|e| e.to_string())?;

    let fees = stmt
        .query_map(params![payment_method_id], |row| {
            Ok(TransactionFee {
                id: row.get(0)?,
                payment_method_id: row.get(1)?,
                description: row.get(2)?,
                fee_percent: row.get(3)?,
                fixed_fee: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(fees)
}

#[tauri::command]
pub fn upsert_transaction_fee(
    id: Option<i64>,
    payment_method_id: i64,
    description: String,
    fee_percent: f64,
    fixed_fee: f64,
    state: State<DbState>,
) -> Result<(), String> {
    let description = description.trim().to_string();
    if description.is_empty() {
        return Err("Descrição é obrigatória".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    match id {
        Some(fee_id) => {
            let rows = conn
                .execute(
                    r#"UPDATE "Transaction_Fees"
                       SET description = ?, fee_percent = ?, fixed_fee = ?, updated_at = CURRENT_TIMESTAMP
                       WHERE id = ?"#,
                    params![description, fee_percent, fixed_fee, fee_id],
                )
                .map_err(|e| e.to_string())?;
            if rows == 0 {
                return Err(format!("Taxa {} não encontrada", fee_id));
            }
        }
        None => {
            conn.execute(
                r#"INSERT INTO "Transaction_Fees"
                   (payment_method_id, description, fee_percent, fixed_fee, updated_at, created_at)
                   VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
                params![payment_method_id, description, fee_percent, fixed_fee],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[cfg(test)]
pub(crate) fn insert_payment_method(conn: &duckdb::Connection, name: &str) -> i64 {
    conn.query_row(
        r#"INSERT INTO "Payment_Methods" (name, active, updated_at, created_at)
           VALUES (?, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
           RETURNING id"#,
        params![name],
        |r| r.get(0),
    ).unwrap()
}

#[cfg(test)]
fn insert_user_local(conn: &duckdb::Connection, name: &str, nickname: &str, active: bool) {
    let pw_hash = bcrypt::hash("senha123", DEFAULT_COST).unwrap();
    conn.execute(
        r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
           VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name, nickname, pw_hash, active],
    )
    .unwrap();
}

#[cfg(test)]
fn insert_business_settings_full(conn: &duckdb::Connection, name: &str, inv: f64, margin: f64) {
    conn.execute(
        r#"INSERT INTO "Business_Settings" (name, investment_rate, profit_margin, updated_at, created_at)
           VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name, inv, margin],
    ).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_db;

    #[test]
    fn update_business_settings_rejects_empty_name() {
        let result = {
            let name = "  ".trim().to_string();
            if name.is_empty() { Err("Nome da empresa não pode ser vazio".to_string()) } else { Ok(()) }
        };
        assert!(result.is_err());
    }

    #[test]
    fn update_business_settings_rejects_rate_out_of_range() {
        let inv: f64 = 1.5;
        let result: Result<(), String> = if !(0.0..=1.0).contains(&inv) {
            Err("Taxa fora do intervalo".to_string())
        } else {
            Ok(())
        };
        assert!(result.is_err());
    }

    #[test]
    fn update_business_settings_persists_new_values() {
        let conn = test_db();
        insert_business_settings_full(&conn, "Empresa Velha", 0.10, 0.20);
        let rows = conn.execute(
            r#"UPDATE "Business_Settings"
               SET name = 'Empresa Nova', investment_rate = 0.15, profit_margin = 0.25,
                   updated_at = CURRENT_TIMESTAMP"#,
            [],
        ).unwrap();
        assert_eq!(rows, 1);
        let name: String = conn.query_row(
            r#"SELECT name FROM "Business_Settings" LIMIT 1"#,
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(name, "Empresa Nova");
    }

    #[test]
    fn list_users_returns_all_users() {
        let conn = test_db();
        insert_user_local(&conn, "Alice", "alice", true);
        insert_user_local(&conn, "Bob", "bob", false);
        let count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Users""#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn change_user_password_rejects_empty() {
        let pw = "  ".trim().to_string();
        assert!(pw.is_empty());
    }

    #[test]
    fn change_user_password_updates_hash() {
        let conn = test_db();
        insert_user_local(&conn, "Carol", "carol", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" WHERE nickname = 'carol'"#, [], |r| r.get(0))
            .unwrap();
        let new_hash = bcrypt::hash("novasenha", DEFAULT_COST).unwrap();
        let rows = conn.execute(
            r#"UPDATE "Users" SET password_hash = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![new_hash, user_id],
        ).unwrap();
        assert_eq!(rows, 1);
        let stored: String = conn
            .query_row(r#"SELECT password_hash FROM "Users" WHERE id = ?"#, params![user_id], |r| r.get(0))
            .unwrap();
        assert!(bcrypt::verify("novasenha", &stored).unwrap());
    }

    #[test]
    fn toggle_user_active_prevents_last_active_deactivation() {
        let conn = test_db();
        insert_user_local(&conn, "Solo", "solo", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" WHERE nickname = 'solo'"#, [], |r| r.get(0))
            .unwrap();
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Users" WHERE active = true AND id != ?"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "should be last active user");
    }

    #[test]
    fn toggle_user_active_deactivates_when_others_exist() {
        let conn = test_db();
        insert_user_local(&conn, "User1", "user1", true);
        insert_user_local(&conn, "User2", "user2", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" WHERE nickname = 'user1'"#, [], |r| r.get(0))
            .unwrap();
        let rows = conn.execute(
            r#"UPDATE "Users" SET active = false, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![user_id],
        ).unwrap();
        assert_eq!(rows, 1);
        let active: bool = conn
            .query_row(r#"SELECT active FROM "Users" WHERE id = ?"#, params![user_id], |r| r.get(0))
            .unwrap();
        assert!(!active);
    }

    #[test]
    fn create_payment_method_rejects_empty_name() {
        let name = "  ".trim().to_string();
        assert!(name.is_empty());
    }

    #[test]
    fn create_payment_method_inserts_active_record() {
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Payment_Methods" (name, active, updated_at, created_at)
               VALUES ('Pix', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();
        let active: bool = conn
            .query_row(r#"SELECT active FROM "Payment_Methods" WHERE name = 'Pix'"#, [], |r| r.get(0))
            .unwrap();
        assert!(active);
    }

    #[test]
    fn list_transaction_fees_returns_fees_for_method() {
        let conn = test_db();
        let pm_id = insert_payment_method(&conn, "Cartão");
        conn.execute(
            r#"INSERT INTO "Transaction_Fees" (payment_method_id, description, fee_percent, fixed_fee, updated_at, created_at)
               VALUES (?, 'Crédito à vista', 0.02, 0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![pm_id],
        ).unwrap();
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Transaction_Fees" WHERE payment_method_id = ?"#,
                params![pm_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn upsert_transaction_fee_inserts_new_fee() {
        let conn = test_db();
        let pm_id = insert_payment_method(&conn, "Débito");
        conn.execute(
            r#"INSERT INTO "Transaction_Fees" (payment_method_id, description, fee_percent, fixed_fee, updated_at, created_at)
               VALUES (?, 'Débito padrão', 0.015, 0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![pm_id],
        ).unwrap();
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Transaction_Fees" WHERE payment_method_id = ?"#,
                params![pm_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        let desc: String = conn
            .query_row(
                r#"SELECT description FROM "Transaction_Fees" WHERE payment_method_id = ?"#,
                params![pm_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(desc, "Débito padrão");
    }
}
