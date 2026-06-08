use duckdb::params;
use serde::Serialize;
use tauri::State;

use crate::db::DbState;

#[derive(Serialize)]
pub struct FinancialEntry {
    pub id: i64,
    pub entry_type: String,
    pub name: String,
    pub category: String,
    pub amount: f64,
    pub is_fixed: bool,
    pub is_recurring: bool,
    pub reference_date: String,
    pub expense_subtype: Option<String>,
}

#[tauri::command]
pub fn list_financial_entries(
    month: String,
    entry_type: String,
    state: State<DbState>,
) -> Result<Vec<FinancialEntry>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, type, name, category, amount, is_fixed, is_recurring,
                   CAST(reference_date AS VARCHAR), expense_subtype
            FROM "Financial_Entries"
            WHERE strftime('%Y-%m', reference_date) = ?
              AND type = ?
            ORDER BY reference_date DESC, created_at DESC
        "#,
        )
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(params![month, entry_type], |row| {
            Ok(FinancialEntry {
                id: row.get(0)?,
                entry_type: row.get(1)?,
                name: row.get(2)?,
                category: row.get(3)?,
                amount: row.get(4)?,
                is_fixed: row.get(5)?,
                is_recurring: row.get(6)?,
                reference_date: row.get(7)?,
                expense_subtype: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[tauri::command]
pub fn list_recurring_suggestions(
    month: String,
    entry_type: String,
    state: State<DbState>,
) -> Result<Vec<FinancialEntry>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    // Entries marked recurring from past months not yet registered in target month
    let mut stmt = conn
        .prepare(
            r#"
            SELECT DISTINCT ON (name, type)
                id, type, name, category, amount, is_fixed, is_recurring,
                CAST(reference_date AS VARCHAR), expense_subtype
            FROM "Financial_Entries"
            WHERE is_recurring = true
              AND type = ?
              AND strftime('%Y-%m', reference_date) < ?
              AND (name, type) NOT IN (
                SELECT name, type FROM "Financial_Entries"
                WHERE strftime('%Y-%m', reference_date) = ?
              )
            ORDER BY name, type, reference_date DESC
        "#,
        )
        .map_err(|e| e.to_string())?;

    let suggestions = stmt
        .query_map(params![entry_type, month, month], |row| {
            Ok(FinancialEntry {
                id: row.get(0)?,
                entry_type: row.get(1)?,
                name: row.get(2)?,
                category: row.get(3)?,
                amount: row.get(4)?,
                is_fixed: row.get(5)?,
                is_recurring: row.get(6)?,
                reference_date: row.get(7)?,
                expense_subtype: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(suggestions)
}

#[tauri::command]
pub fn create_financial_entry(
    name: String,
    category: String,
    amount: f64,
    entry_type: String,
    is_fixed: bool,
    is_recurring: bool,
    reference_date: String,
    expense_subtype: Option<String>,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    let category = category.trim().to_string();
    let reference_date = reference_date.trim().to_string();

    if name.is_empty() {
        return Err("Nome é obrigatório".to_string());
    }
    if amount <= 0.0 {
        return Err("Valor deve ser maior que zero".to_string());
    }
    if reference_date.is_empty() {
        return Err("Data de referência é obrigatória".to_string());
    }
    if entry_type != "income" && entry_type != "expense" {
        return Err("Tipo inválido".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        r#"INSERT INTO "Financial_Entries"
           (type, name, category, amount, is_fixed, is_recurring, expense_subtype,
            reference_date, updated_at, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, CAST(? AS DATE), CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![
            entry_type,
            name,
            category,
            amount,
            is_fixed,
            is_recurring,
            expense_subtype,
            reference_date
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_financial_entry(
    id: i64,
    name: String,
    category: String,
    amount: f64,
    is_fixed: bool,
    is_recurring: bool,
    reference_date: String,
    expense_subtype: Option<String>,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    let category = category.trim().to_string();
    let reference_date = reference_date.trim().to_string();

    if name.is_empty() {
        return Err("Nome é obrigatório".to_string());
    }
    if amount <= 0.0 {
        return Err("Valor deve ser maior que zero".to_string());
    }
    if reference_date.is_empty() {
        return Err("Data de referência é obrigatória".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let rows = conn
        .execute(
            r#"UPDATE "Financial_Entries"
               SET name = ?, category = ?, amount = ?, is_fixed = ?, is_recurring = ?,
                   expense_subtype = ?,
                   reference_date = CAST(? AS DATE), updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![
                name,
                category,
                amount,
                is_fixed,
                is_recurring,
                expense_subtype,
                reference_date,
                id
            ],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Lançamento {} não encontrado", id));
    }

    Ok(())
}

#[tauri::command]
pub fn delete_financial_entry(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let rows = conn
        .execute(
            r#"DELETE FROM "Financial_Entries" WHERE id = ?"#,
            params![id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Lançamento {} não encontrado", id));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_db;
    use duckdb::Connection;

    fn insert_financial_entry(
        conn: &Connection,
        name: &str,
        entry_type: &str,
        amount: f64,
        is_fixed: bool,
        is_recurring: bool,
        reference_date: &str,
    ) -> i64 {
        conn.execute(
            r#"INSERT INTO "Financial_Entries"
               (type, name, category, amount, is_fixed, is_recurring, reference_date,
                updated_at, created_at)
               VALUES (?, ?, 'geral', ?, ?, ?, CAST(? AS DATE), CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![entry_type, name, amount, is_fixed, is_recurring, reference_date],
        ).unwrap();
        conn.query_row(
            r#"SELECT id FROM "Financial_Entries" WHERE name = ? ORDER BY created_at DESC LIMIT 1"#,
            params![name],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn create_financial_entry_rejects_empty_name() {
        let name = "  ".trim().to_string();
        assert!(name.is_empty());
    }

    #[test]
    fn create_financial_entry_rejects_zero_amount() {
        assert!(0.0_f64 <= 0.0);
    }

    #[test]
    fn list_financial_entries_filters_by_month_and_type() {
        let conn = test_db();
        insert_financial_entry(
            &conn,
            "Aluguel",
            "expense",
            1200.0,
            true,
            true,
            "2026-05-01",
        );
        insert_financial_entry(
            &conn,
            "Energia",
            "expense",
            300.0,
            true,
            false,
            "2026-05-15",
        );
        insert_financial_entry(
            &conn,
            "Venda extra",
            "income",
            500.0,
            false,
            false,
            "2026-05-10",
        );
        insert_financial_entry(
            &conn,
            "Aluguel",
            "expense",
            1200.0,
            true,
            true,
            "2026-04-01",
        );

        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Financial_Entries"
               WHERE strftime('%Y-%m', reference_date) = '2026-05' AND type = 'expense'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        let income_count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Financial_Entries"
               WHERE strftime('%Y-%m', reference_date) = '2026-05' AND type = 'income'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(income_count, 1);
    }

    #[test]
    fn list_recurring_suggestions_returns_missing_entries_for_month() {
        let conn = test_db();
        insert_financial_entry(
            &conn,
            "Aluguel",
            "expense",
            1200.0,
            true,
            true,
            "2026-04-01",
        );
        insert_financial_entry(&conn, "Energia", "expense", 300.0, true, true, "2026-04-01");
        insert_financial_entry(&conn, "Energia", "expense", 320.0, true, true, "2026-05-01");

        let suggestion_count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM (
                SELECT DISTINCT ON (name, type) name, type
                FROM "Financial_Entries"
                WHERE is_recurring = true
                  AND type = 'expense'
                  AND strftime('%Y-%m', reference_date) < '2026-05'
                  AND (name, type) NOT IN (
                    SELECT name, type FROM "Financial_Entries"
                    WHERE strftime('%Y-%m', reference_date) = '2026-05'
                  )
                ORDER BY name, type, reference_date DESC
            )"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(suggestion_count, 1);
    }

    #[test]
    fn delete_financial_entry_removes_record() {
        let conn = test_db();
        let id = insert_financial_entry(&conn, "Taxa", "expense", 50.0, false, false, "2026-05-01");
        let rows = conn
            .execute(
                r#"DELETE FROM "Financial_Entries" WHERE id = ?"#,
                params![id],
            )
            .unwrap();
        assert_eq!(rows, 1);
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Financial_Entries" WHERE id = ?"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }
}
