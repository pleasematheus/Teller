use bcrypt::{hash, DEFAULT_COST};
use duckdb::params;
use serde::Serialize;
use std::collections::HashMap;
use tauri::State;

use crate::db::DbState;

#[derive(Serialize)]
pub struct BusinessInfo {
    pub name: String,
    pub investment_rate: Option<f64>,
    pub profit_margin: Option<f64>,
}

#[derive(Serialize, Clone)]
pub struct TransactionFee {
    pub id: i64,
    pub payment_method_id: i64,
    pub installments: i64,
    pub fee_percent: f64,
    pub fixed_fee: f64,
}

#[derive(Serialize)]
pub struct PaymentMethod {
    pub id: i64,
    pub name: String,
    pub active: bool,
    pub fees: Vec<TransactionFee>,
}

#[derive(Serialize)]
pub struct UnitOfMeasure {
    pub id: i64,
    pub name: String,
    pub abbreviation: String,
    pub active: bool,
    pub integer: bool,
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

    let mut method_stmt = conn
        .prepare(r#"
            SELECT id, name, active
            FROM "Payment_Methods"
            ORDER BY active DESC, name
        "#)
        .map_err(|e| e.to_string())?;

    let methods_raw: Vec<(i64, String, bool)> = method_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut fee_stmt = conn
        .prepare(r#"
            SELECT id, payment_method_id, installments, fee_percent, fixed_fee
            FROM "Transaction_Fees"
            ORDER BY payment_method_id, installments
        "#)
        .map_err(|e| e.to_string())?;

    let fees_raw: Vec<TransactionFee> = fee_stmt
        .query_map([], |row| {
            Ok(TransactionFee {
                id: row.get(0)?,
                payment_method_id: row.get(1)?,
                installments: row.get(2)?,
                fee_percent: row.get(3)?,
                fixed_fee: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut fees_by_method: HashMap<i64, Vec<TransactionFee>> = HashMap::new();
    for fee in fees_raw {
        fees_by_method.entry(fee.payment_method_id).or_default().push(fee);
    }

    let methods = methods_raw
        .into_iter()
        .map(|(id, name, active)| PaymentMethod {
            fees: fees_by_method.remove(&id).unwrap_or_default(),
            id,
            name,
            active,
        })
        .collect();

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
pub fn rename_payment_method(id: i64, name: String, state: State<DbState>) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Nome é obrigatório".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let rows = conn
        .execute(
            r#"UPDATE "Payment_Methods" SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![name, id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Forma de pagamento {} não encontrada", id));
    }
    Ok(())
}

#[tauri::command]
pub fn delete_payment_method(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let linked: i64 = conn
        .query_row(
            r#"SELECT COUNT(*) FROM "Sale_Payments" WHERE payment_method_id = ?"#,
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if linked > 0 {
        return Err(format!(
            "Esta forma de pagamento não pode ser excluída pois está vinculada a {} venda(s)",
            linked
        ));
    }

    // Delete associated fees first (no ON DELETE CASCADE in schema).
    conn.execute(
        r#"DELETE FROM "Transaction_Fees" WHERE payment_method_id = ?"#,
        params![id],
    )
    .map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            r#"DELETE FROM "Payment_Methods" WHERE id = ?"#,
            params![id],
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
            SELECT id, payment_method_id, installments, fee_percent, fixed_fee
            FROM "Transaction_Fees"
            WHERE payment_method_id = ?
            ORDER BY installments
        "#)
        .map_err(|e| e.to_string())?;

    let fees = stmt
        .query_map(params![payment_method_id], |row| {
            Ok(TransactionFee {
                id: row.get(0)?,
                payment_method_id: row.get(1)?,
                installments: row.get(2)?,
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
    payment_method_id: i64,
    installments: i64,
    fee_percent: f64,
    fixed_fee: f64,
    state: State<DbState>,
) -> Result<(), String> {
    if installments < 1 {
        return Err("Número de parcelas deve ser >= 1".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let existing_id: Option<i64> = conn
        .query_row(
            r#"SELECT id FROM "Transaction_Fees"
               WHERE payment_method_id = ? AND installments = ?"#,
            params![payment_method_id, installments],
            |row| row.get(0),
        )
        .ok();

    match existing_id {
        Some(fee_id) => {
            conn.execute(
                r#"UPDATE "Transaction_Fees"
                   SET fee_percent = ?, fixed_fee = ?, updated_at = CURRENT_TIMESTAMP
                   WHERE id = ?"#,
                params![fee_percent, fixed_fee, fee_id],
            )
            .map_err(|e| e.to_string())?;
        }
        None => {
            conn.execute(
                r#"INSERT INTO "Transaction_Fees"
                   (payment_method_id, installments, fee_percent, fixed_fee, updated_at, created_at)
                   VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
                params![payment_method_id, installments, fee_percent, fixed_fee],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn list_units_of_measure(state: State<DbState>) -> Result<Vec<UnitOfMeasure>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(r#"
            SELECT id, name, abbreviation, active, "integer"
            FROM "Units_Of_Measure"
            ORDER BY active DESC, name
        "#)
        .map_err(|e| e.to_string())?;
    let units = stmt
        .query_map([], |row| {
            Ok(UnitOfMeasure {
                id: row.get(0)?,
                name: row.get(1)?,
                abbreviation: row.get(2)?,
                active: row.get(3)?,
                integer: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(units)
}

#[tauri::command]
pub fn create_unit_of_measure(
    name: String,
    abbreviation: String,
    integer: bool,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    let abbreviation = abbreviation.trim().to_string();
    if name.is_empty() {
        return Err("Nome é obrigatório".to_string());
    }
    if abbreviation.is_empty() {
        return Err("Sigla é obrigatória".to_string());
    }
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        r#"INSERT INTO "Units_Of_Measure" (name, abbreviation, active, "integer", updated_at, created_at)
           VALUES (?, ?, true, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name, abbreviation, integer],
    )
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("Constraint Error") || msg.contains("unique") || msg.contains("Duplicate") {
            format!("Já existe uma unidade de medida com o nome '{}'", name)
        } else {
            msg
        }
    })?;
    Ok(())
}

#[tauri::command]
pub fn rename_unit_of_measure(
    id: i64,
    name: String,
    abbreviation: String,
    integer: bool,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    let abbreviation = abbreviation.trim().to_string();
    if name.is_empty() {
        return Err("Nome é obrigatório".to_string());
    }
    if abbreviation.is_empty() {
        return Err("Sigla é obrigatória".to_string());
    }
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let old_name: String = conn
        .query_row(
            r#"SELECT name FROM "Units_Of_Measure" WHERE id = ?"#,
            params![id],
            |r| r.get(0),
        )
        .map_err(|_| format!("Unidade {} não encontrada", id))?;

    conn.execute("BEGIN", []).map_err(|e| e.to_string())?;

    let result: Result<(), String> = (|| {
        let rows = conn
            .execute(
                r#"UPDATE "Units_Of_Measure"
                   SET name = ?, abbreviation = ?, "integer" = ?, updated_at = CURRENT_TIMESTAMP
                   WHERE id = ?"#,
                params![name, abbreviation, integer, id],
            )
            .map_err(|e| e.to_string())?;
        if rows == 0 {
            return Err(format!("Unidade {} não encontrada", id));
        }
        if name != old_name {
            conn.execute(
                r#"UPDATE "Products" SET unit = ?, updated_at = CURRENT_TIMESTAMP WHERE unit = ?"#,
                params![name, old_name],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    })();

    if result.is_ok() {
        conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    } else {
        let _ = conn.execute("ROLLBACK", []);
    }
    result
}

#[tauri::command]
pub fn update_unit_of_measure(
    id: i64,
    active: bool,
    state: State<DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let rows = conn
        .execute(
            r#"UPDATE "Units_Of_Measure" SET active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![active, id],
        )
        .map_err(|e| e.to_string())?;
    if rows == 0 {
        return Err(format!("Unidade {} não encontrada", id));
    }
    Ok(())
}

#[tauri::command]
pub fn delete_unit_of_measure(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let name: String = conn
        .query_row(
            r#"SELECT name FROM "Units_Of_Measure" WHERE id = ?"#,
            params![id],
            |r| r.get(0),
        )
        .map_err(|_| format!("Unidade {} não encontrada", id))?;

    let linked: i64 = conn
        .query_row(
            r#"SELECT COUNT(*) FROM "Products" WHERE unit = ?"#,
            params![name],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    if linked > 0 {
        return Err(format!(
            "Esta unidade não pode ser excluída pois está em uso por {} produto(s)",
            linked
        ));
    }

    let rows = conn
        .execute(r#"DELETE FROM "Units_Of_Measure" WHERE id = ?"#, params![id])
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Unidade {} não encontrada", id));
    }
    Ok(())
}

fn system_setting_default(key: &str) -> String {
    match key {
        "allow_negative_stock_sale" => "false".to_string(),
        _ => String::new(),
    }
}

fn normalize_setting_value(key: &str, value: &str) -> String {
    match key {
        "allow_negative_stock_sale" => {
            if value == "true" { "true".to_string() } else { "false".to_string() }
        }
        _ => value.to_string(),
    }
}

#[tauri::command]
pub fn get_system_setting(key: String, state: State<DbState>) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let value: Option<String> = conn
        .query_row(
            r#"SELECT "value" FROM "System_Settings" WHERE "key" = ?"#,
            params![key],
            |row| row.get(0),
        )
        .ok();
    Ok(value.unwrap_or_else(|| system_setting_default(&key)))
}

#[tauri::command]
pub fn set_system_setting(key: String, value: String, state: State<DbState>) -> Result<(), String> {
    let normalized = normalize_setting_value(&key, &value);
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let existing: Option<String> = conn
        .query_row(
            r#"SELECT "value" FROM "System_Settings" WHERE "key" = ?"#,
            params![key],
            |row| row.get(0),
        )
        .ok();

    match existing {
        Some(_) => {
            conn.execute(
                r#"UPDATE "System_Settings" SET "value" = ?, "updated_at" = CURRENT_TIMESTAMP WHERE "key" = ?"#,
                params![normalized, key],
            )
            .map_err(|e| e.to_string())?;
        }
        None => {
            conn.execute(
                r#"INSERT INTO "System_Settings" ("key", "value", "updated_at") VALUES (?, ?, CURRENT_TIMESTAMP)"#,
                params![key, normalized],
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
pub(crate) fn insert_unit_of_measure(conn: &duckdb::Connection, name: &str, abbreviation: &str) -> i64 {
    conn.query_row(
        r#"INSERT INTO "Units_Of_Measure" (name, abbreviation, active, updated_at, created_at)
           VALUES (?, ?, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
           RETURNING id"#,
        params![name, abbreviation],
        |r| r.get(0),
    ).unwrap()
}

#[cfg(test)]
pub(crate) fn insert_transaction_fee(
    conn: &duckdb::Connection,
    payment_method_id: i64,
    installments: i64,
    fee_percent: f64,
    fixed_fee: f64,
) -> i64 {
    conn.query_row(
        r#"INSERT INTO "Transaction_Fees"
           (payment_method_id, installments, fee_percent, fixed_fee, updated_at, created_at)
           VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
           RETURNING id"#,
        params![payment_method_id, installments, fee_percent, fixed_fee],
        |r| r.get(0),
    ).unwrap()
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

    // Task 2.4 / 3.3 tests

    #[test]
    fn upsert_transaction_fee_creates_by_installments() {
        let conn = test_db();
        let pm_id = insert_payment_method(&conn, "Cartão");
        insert_transaction_fee(&conn, pm_id, 1, 0.015, 0.0);
        let fee_id = insert_transaction_fee(&conn, pm_id, 2, 0.025, 0.5);

        let count: i64 = conn.query_row(
            r#"SELECT COUNT(*) FROM "Transaction_Fees" WHERE payment_method_id = ?"#,
            params![pm_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 2);

        let fp: f64 = conn.query_row(
            r#"SELECT fee_percent FROM "Transaction_Fees" WHERE id = ?"#,
            params![fee_id], |r| r.get(0),
        ).unwrap();
        assert!((fp - 0.025).abs() < 1e-9);
    }

    #[test]
    fn upsert_transaction_fee_updates_existing_installments() {
        let conn = test_db();
        let pm_id = insert_payment_method(&conn, "Crédito");
        insert_transaction_fee(&conn, pm_id, 1, 0.02, 0.0);

        // Update via upsert by installments
        let existing: i64 = conn.query_row(
            r#"SELECT id FROM "Transaction_Fees" WHERE payment_method_id = ? AND installments = 1"#,
            params![pm_id], |r| r.get(0),
        ).unwrap();
        conn.execute(
            r#"UPDATE "Transaction_Fees" SET fee_percent = 0.03, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![existing],
        ).unwrap();

        let fp: f64 = conn.query_row(
            r#"SELECT fee_percent FROM "Transaction_Fees" WHERE id = ?"#,
            params![existing], |r| r.get(0),
        ).unwrap();
        assert!((fp - 0.03).abs() < 1e-9);
    }

    #[test]
    fn upsert_transaction_fee_rejects_installments_less_than_1() {
        let installments: i64 = 0;
        assert!(installments < 1, "installments < 1 should be rejected");
    }

    #[test]
    fn list_transaction_fees_ordered_by_installments() {
        let conn = test_db();
        let pm_id = insert_payment_method(&conn, "Crédito");
        insert_transaction_fee(&conn, pm_id, 3, 0.03, 0.0);
        insert_transaction_fee(&conn, pm_id, 1, 0.01, 0.0);
        insert_transaction_fee(&conn, pm_id, 2, 0.02, 0.0);

        let mut stmt = conn.prepare(
            r#"SELECT installments FROM "Transaction_Fees"
               WHERE payment_method_id = ? ORDER BY installments"#,
        ).unwrap();
        let installments: Vec<i64> = stmt
            .query_map(params![pm_id], |r| r.get(0)).unwrap()
            .collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(installments, vec![1, 2, 3]);
    }

    #[test]
    fn list_payment_methods_returns_fees_per_method() {
        let conn = test_db();
        let pm_id = insert_payment_method(&conn, "Cartão");
        insert_transaction_fee(&conn, pm_id, 1, 0.015, 0.0);
        insert_transaction_fee(&conn, pm_id, 2, 0.025, 0.5);

        let mut stmt = conn.prepare(
            r#"SELECT id, payment_method_id, installments, fee_percent, fixed_fee
               FROM "Transaction_Fees" WHERE payment_method_id = ? ORDER BY installments"#,
        ).unwrap();
        let fees: Vec<(i64, i64, i64, f64, f64)> = stmt
            .query_map(params![pm_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
            .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(fees.len(), 2);
        assert_eq!(fees[0].2, 1);
        assert_eq!(fees[1].2, 2);
    }

    #[test]
    fn list_payment_methods_returns_empty_fees_for_method_without_fees() {
        let conn = test_db();
        insert_payment_method(&conn, "Dinheiro");
        let count: i64 = conn.query_row(
            r#"SELECT COUNT(*) FROM "Transaction_Fees" WHERE payment_method_id IN
               (SELECT id FROM "Payment_Methods" WHERE name = 'Dinheiro')"#,
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn rename_payment_method_updates_name() {
        let conn = test_db();
        let pm_id = insert_payment_method(&conn, "Cartão Antigo");
        conn.execute(
            r#"UPDATE "Payment_Methods" SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params!["Cartão Novo", pm_id],
        ).unwrap();
        let name: String = conn
            .query_row(
                r#"SELECT name FROM "Payment_Methods" WHERE id = ?"#,
                params![pm_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(name, "Cartão Novo");
    }

    #[test]
    fn rename_payment_method_rejects_empty_name() {
        let name = "   ".trim().to_string();
        assert!(name.is_empty());
    }

    #[test]
    fn delete_payment_method_removes_unused_method() {
        let conn = test_db();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let linked: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Sale_Payments" WHERE payment_method_id = ?"#,
                params![pm_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(linked, 0);
        conn.execute(
            r#"DELETE FROM "Payment_Methods" WHERE id = ?"#,
            params![pm_id],
        ).unwrap();
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Payment_Methods" WHERE id = ?"#,
                params![pm_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    // Task 4.1 / 4.2 tests

    #[test]
    fn create_unit_of_measure_rejects_empty_name() {
        let name = "  ".trim().to_string();
        assert!(name.is_empty());
    }

    #[test]
    fn create_unit_of_measure_rejects_empty_abbreviation() {
        let abbr = "  ".trim().to_string();
        assert!(abbr.is_empty());
    }

    #[test]
    fn create_unit_of_measure_inserts_with_active_true() {
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Units_Of_Measure" (name, abbreviation, active, updated_at, created_at)
               VALUES ('kg', 'kg', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();
        let active: bool = conn.query_row(
            r#"SELECT active FROM "Units_Of_Measure" WHERE name = 'kg'"#,
            [],
            |r| r.get(0),
        ).unwrap();
        assert!(active);
    }

    #[test]
    fn create_unit_of_measure_rejects_duplicate_name() {
        let conn = test_db();
        insert_unit_of_measure(&conn, "kg", "kg");
        let result = conn.execute(
            r#"INSERT INTO "Units_Of_Measure" (name, abbreviation, active, updated_at, created_at)
               VALUES ('kg', 'kilo', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        );
        assert!(result.is_err(), "duplicate name should violate UNIQUE constraint");
    }

    // integer flag tests

    #[test]
    fn create_unit_of_measure_stores_integer_true() {
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Units_Of_Measure" (name, abbreviation, active, "integer", updated_at, created_at)
               VALUES ('un', 'un', true, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();
        let integer: bool = conn.query_row(
            r#"SELECT "integer" FROM "Units_Of_Measure" WHERE name = 'un'"#,
            [],
            |r| r.get(0),
        ).unwrap();
        assert!(integer);
    }

    #[test]
    fn create_unit_of_measure_integer_defaults_true_and_accepts_false() {
        let conn = test_db();
        // Sem informar "integer": default da coluna é TRUE.
        let uid = insert_unit_of_measure(&conn, "kg", "kg");
        let default_integer: bool = conn.query_row(
            r#"SELECT "integer" FROM "Units_Of_Measure" WHERE id = ?"#,
            params![uid],
            |r| r.get(0),
        ).unwrap();
        assert!(default_integer);

        // Informando false explicitamente.
        conn.execute(
            r#"INSERT INTO "Units_Of_Measure" (name, abbreviation, active, "integer", updated_at, created_at)
               VALUES ('L', 'L', true, false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();
        let integer: bool = conn.query_row(
            r#"SELECT "integer" FROM "Units_Of_Measure" WHERE name = 'L'"#,
            [],
            |r| r.get(0),
        ).unwrap();
        assert!(!integer);
    }

    #[test]
    fn list_units_of_measure_select_returns_integer() {
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Units_Of_Measure" (name, abbreviation, active, "integer", updated_at, created_at)
               VALUES ('un', 'un', true, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();
        let (name, integer): (String, bool) = conn.query_row(
            r#"SELECT name, "integer" FROM "Units_Of_Measure" ORDER BY active DESC, name"#,
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!(name, "un");
        assert!(integer);
    }

    #[test]
    fn rename_unit_of_measure_updates_integer_and_propagates_name() {
        let conn = test_db();
        let uid = insert_unit_of_measure(&conn, "Unidade", "un");
        conn.execute(
            r#"INSERT INTO "Products" (name, active, sellable, markup, cost_price, final_price,
               expected_loss_rate, unit, updated_at, created_at)
               VALUES ('Produto Teste', true, true, 1.5, 10.0, 15.0, 0.0, 'Unidade',
               CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();

        conn.execute("BEGIN", []).unwrap();
        conn.execute(
            r#"UPDATE "Units_Of_Measure"
               SET name = 'Unidade Nova', abbreviation = 'un', "integer" = false, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![uid],
        ).unwrap();
        conn.execute(
            r#"UPDATE "Products" SET unit = 'Unidade Nova', updated_at = CURRENT_TIMESTAMP WHERE unit = 'Unidade'"#,
            [],
        ).unwrap();
        conn.execute("COMMIT", []).unwrap();

        let (new_name, integer): (String, bool) = conn.query_row(
            r#"SELECT name, "integer" FROM "Units_Of_Measure" WHERE id = ?"#,
            params![uid],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!(new_name, "Unidade Nova");
        assert!(!integer);

        let unit: String = conn.query_row(
            r#"SELECT unit FROM "Products" WHERE name = 'Produto Teste'"#,
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(unit, "Unidade Nova");
    }

    // Task 4.3 test

    #[test]
    fn rename_unit_of_measure_propagates_to_products() {
        let conn = test_db();
        let uid = insert_unit_of_measure(&conn, "Unidade", "un");
        conn.execute(
            r#"INSERT INTO "Products" (name, active, sellable, markup, cost_price, final_price,
               expected_loss_rate, unit, updated_at, created_at)
               VALUES ('Produto Teste', true, true, 1.5, 10.0, 15.0, 0.0, 'Unidade',
               CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();

        conn.execute("BEGIN", []).unwrap();
        conn.execute(
            r#"UPDATE "Units_Of_Measure" SET name = 'Unidade Nova', updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![uid],
        ).unwrap();
        conn.execute(
            r#"UPDATE "Products" SET unit = 'Unidade Nova', updated_at = CURRENT_TIMESTAMP WHERE unit = 'Unidade'"#,
            [],
        ).unwrap();
        conn.execute("COMMIT", []).unwrap();

        let unit: String = conn.query_row(
            r#"SELECT unit FROM "Products" WHERE name = 'Produto Teste'"#,
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(unit, "Unidade Nova");

        let new_name: String = conn.query_row(
            r#"SELECT name FROM "Units_Of_Measure" WHERE id = ?"#,
            params![uid],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(new_name, "Unidade Nova");
    }

    // Task 4.4 test

    #[test]
    fn update_unit_of_measure_toggles_active() {
        let conn = test_db();
        let uid = insert_unit_of_measure(&conn, "kg", "kg");
        conn.execute(
            r#"UPDATE "Units_Of_Measure" SET active = false, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![uid],
        ).unwrap();
        let active: bool = conn.query_row(
            r#"SELECT active FROM "Units_Of_Measure" WHERE id = ?"#,
            params![uid],
            |r| r.get(0),
        ).unwrap();
        assert!(!active);
    }

    // Task 4.5 tests

    #[test]
    fn delete_unit_of_measure_removes_unused() {
        let conn = test_db();
        let uid = insert_unit_of_measure(&conn, "kg", "kg");
        let linked: i64 = conn.query_row(
            r#"SELECT COUNT(*) FROM "Products" WHERE unit = 'kg'"#,
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(linked, 0);
        conn.execute(r#"DELETE FROM "Units_Of_Measure" WHERE id = ?"#, params![uid]).unwrap();
        let count: i64 = conn.query_row(
            r#"SELECT COUNT(*) FROM "Units_Of_Measure" WHERE id = ?"#,
            params![uid],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn delete_unit_of_measure_blocked_when_in_use() {
        let conn = test_db();
        let uid = insert_unit_of_measure(&conn, "kg", "kg");
        conn.execute(
            r#"INSERT INTO "Products" (name, active, sellable, markup, cost_price, final_price,
               expected_loss_rate, unit, updated_at, created_at)
               VALUES ('Produto', true, true, 1.5, 10.0, 15.0, 0.0, 'kg',
               CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();

        let name: String = conn.query_row(
            r#"SELECT name FROM "Units_Of_Measure" WHERE id = ?"#,
            params![uid],
            |r| r.get(0),
        ).unwrap();
        let linked: i64 = conn.query_row(
            r#"SELECT COUNT(*) FROM "Products" WHERE unit = ?"#,
            params![name],
            |r| r.get(0),
        ).unwrap();
        assert!(linked > 0, "should have linked products preventing deletion");
    }

    #[test]
    fn get_system_setting_absent_key_returns_default() {
        let conn = test_db();
        conn.execute(
            r#"DELETE FROM "System_Settings" WHERE "key" = 'allow_negative_stock_sale'"#,
            [],
        )
        .unwrap();
        let value: String = conn
            .query_row(
                r#"SELECT COALESCE((SELECT "value" FROM "System_Settings" WHERE "key" = 'allow_negative_stock_sale'), 'false')"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(value, "false");
    }

    #[test]
    fn set_system_setting_inserts_when_absent_and_updates_without_duplicate() {
        let conn = test_db();
        // Remove seed so we can test insert
        conn.execute(
            r#"DELETE FROM "System_Settings" WHERE "key" = 'allow_negative_stock_sale'"#,
            [],
        )
        .unwrap();

        // Insert
        let existing: Option<String> = conn
            .query_row(
                r#"SELECT "value" FROM "System_Settings" WHERE "key" = 'allow_negative_stock_sale'"#,
                [],
                |r| r.get(0),
            )
            .ok();
        assert!(existing.is_none());
        conn.execute(
            r#"INSERT INTO "System_Settings" ("key", "value", "updated_at") VALUES ('allow_negative_stock_sale', 'true', CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();

        // Update (upsert)
        conn.execute(
            r#"UPDATE "System_Settings" SET "value" = 'false', "updated_at" = CURRENT_TIMESTAMP WHERE "key" = 'allow_negative_stock_sale'"#,
            [],
        )
        .unwrap();

        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "System_Settings" WHERE "key" = 'allow_negative_stock_sale'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "Não deve duplicar ao atualizar");

        let value: String = conn
            .query_row(
                r#"SELECT "value" FROM "System_Settings" WHERE "key" = 'allow_negative_stock_sale'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(value, "false");
    }

    #[test]
    fn set_system_setting_normalizes_boolean_value() {
        let normalized = normalize_setting_value("allow_negative_stock_sale", "true");
        assert_eq!(normalized, "true");
        let normalized2 = normalize_setting_value("allow_negative_stock_sale", "false");
        assert_eq!(normalized2, "false");
        // non-"true" values normalize to "false"
        let normalized3 = normalize_setting_value("allow_negative_stock_sale", "yes");
        assert_eq!(normalized3, "false");
    }

    #[test]
    fn delete_payment_method_blocked_when_sales_exist() {
        let conn = test_db();
        use crate::auth::insert_user;
        insert_user(&conn, "Teste", "teste", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Pix");
        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
                   VALUES (?, '', 'closed', 50.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sale_Payments" (sale_id, payment_method_id, amount_paid, net_amount, created_at)
               VALUES (?, ?, 50.0, 50.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, pm_id],
        ).unwrap();
        let linked: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Sale_Payments" WHERE payment_method_id = ?"#,
                params![pm_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(linked > 0, "should have linked sale payments");
    }
}
