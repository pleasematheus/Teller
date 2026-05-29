use duckdb::{params, Connection};
use serde::Serialize;
use tauri::State;

use crate::db::DbState;

#[derive(Serialize)]
pub struct PricingRates {
    pub cv: f64,
    pub df: f64,
    pub investment_rate: f64,
    pub profit_margin: f64,
    pub no_sales_history: bool,
}

#[derive(Serialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
    pub sellable: bool,
    pub cost_price: f64,
    pub markup: f64,
    pub final_price: f64,
    pub expected_loss_rate: f64,
    pub unit: String,
    pub profit_margin: f64,
}

pub fn validate_product(name: &str, cost_price: f64, expected_loss_rate: f64) -> Result<(), String> {
    if name.is_empty() {
        return Err("Nome é obrigatório".to_string());
    }
    if cost_price < 0.0 {
        return Err("Custo não pode ser negativo".to_string());
    }
    if !(0.0..1.0).contains(&expected_loss_rate) {
        return Err("Taxa de perda deve estar entre 0% e 99%".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn get_pricing_rates(state: State<DbState>) -> Result<PricingRates, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let (investment_rate, profit_margin): (f64, f64) = conn
        .query_row(
            r#"SELECT investment_rate, profit_margin FROM "Business_Settings" LIMIT 1"#,
            [],
            |row| {
                let ir: Option<f64> = row.get(0)?;
                let pm: Option<f64> = row.get(1)?;
                Ok((ir.unwrap_or(0.0), pm.unwrap_or(0.0)))
            },
        )
        .unwrap_or((0.0, 0.0));

    let faturamento_ref: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(total_value), 0.0) FROM "Sales"
               WHERE created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    if faturamento_ref == 0.0 {
        return Ok(PricingRates {
            cv: 0.0,
            df: 0.0,
            investment_rate,
            profit_margin,
            no_sales_history: true,
        });
    }

    let cv_total: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(amount), 0.0) FROM "Financial_Entries" WHERE is_fixed = false"#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let df_total: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(amount), 0.0) FROM "Financial_Entries"
               WHERE "type" = 'expense' AND is_fixed = true"#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    Ok(PricingRates {
        cv: cv_total / faturamento_ref,
        df: df_total / faturamento_ref,
        investment_rate,
        profit_margin,
        no_sales_history: false,
    })
}

#[tauri::command]
pub fn get_product_loss_rate(product_id: i64, state: State<DbState>) -> Result<f64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let total_in: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'in'"#,
            params![product_id],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    if total_in == 0.0 {
        return Ok(0.0);
    }

    let total_loss: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out' AND motive = 'perda'"#,
            params![product_id],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    Ok(total_loss / total_in)
}

#[tauri::command]
pub fn get_realized_margin(state: State<DbState>) -> Result<f64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let margin: f64 = conn
        .query_row(
            r#"SELECT COALESCE(
                AVG(
                    (sp.discounted_price - p.cost_price / (1.0 - p.expected_loss_rate))
                    / NULLIF(sp.discounted_price, 0)
                ), 0.0
            )
            FROM "Sales_Products" sp
            JOIN "Products" p ON p.id = sp.product_id
            JOIN "Sales" s ON s.id = sp.sale_id
            WHERE s.status = 'closed'
            AND s.created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    Ok(margin)
}

#[tauri::command]
pub fn list_products(state: State<DbState>) -> Result<Vec<Product>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT id, name, description, active, sellable,
                   cost_price, markup, final_price, expected_loss_rate, unit, profit_margin
            FROM "Products"
            ORDER BY name
        "#)
        .map_err(|e| e.to_string())?;

    let products = stmt
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                active: row.get(3)?,
                sellable: row.get(4)?,
                cost_price: row.get(5)?,
                markup: row.get(6)?,
                final_price: row.get(7)?,
                expected_loss_rate: row.get(8)?,
                unit: row.get(9)?,
                profit_margin: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(products)
}

#[tauri::command]
pub fn create_product(
    name: String,
    description: Option<String>,
    active: bool,
    sellable: bool,
    cost_price: f64,
    markup: f64,
    final_price: f64,
    expected_loss_rate: f64,
    unit: String,
    profit_margin: f64,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    validate_product(&name, cost_price, expected_loss_rate)?;

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        r#"INSERT INTO "Products"
           (name, description, active, sellable, cost_price, markup, final_price,
            expected_loss_rate, unit, profit_margin, updated_at, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name, description, active, sellable, cost_price, markup, final_price,
                expected_loss_rate, unit, profit_margin],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_product(
    id: i64,
    name: String,
    description: Option<String>,
    active: bool,
    sellable: bool,
    cost_price: f64,
    markup: f64,
    final_price: f64,
    expected_loss_rate: f64,
    unit: String,
    profit_margin: f64,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    validate_product(&name, cost_price, expected_loss_rate)?;

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let rows_affected = conn
        .execute(
            r#"UPDATE "Products"
               SET name = ?, description = ?, active = ?, sellable = ?,
                   cost_price = ?, markup = ?, final_price = ?, expected_loss_rate = ?,
                   unit = ?, profit_margin = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![name, description, active, sellable, cost_price, markup, final_price,
                    expected_loss_rate, unit, profit_margin, id],
        )
        .map_err(|e| e.to_string())?;

    if rows_affected == 0 {
        return Err(format!("Produto {} não encontrado", id));
    }

    Ok(())
}

#[cfg(test)]
pub(crate) fn insert_product(conn: &Connection, name: &str, sellable: bool) -> i64 {
    conn.execute(
        r#"INSERT INTO "Products"
           (name, active, sellable, markup, cost_price, final_price, expected_loss_rate,
            updated_at, created_at)
           VALUES (?, true, ?, 1.5, 10.0, 15.0, 0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name, sellable],
    )
    .unwrap();
    conn.query_row(
        r#"SELECT id FROM "Products" WHERE name = ? LIMIT 1"#,
        params![name],
        |r| r.get(0),
    )
    .unwrap()
}

#[cfg(test)]
pub(crate) fn insert_business_settings(conn: &Connection, name: &str) {
    conn.execute(
        r#"INSERT INTO "Business_Settings" (name, investment_rate, profit_margin, updated_at, created_at)
           VALUES (?, 0.1, 0.2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name],
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_db;
    use duckdb::params;

    #[test]
    fn validate_product_rejects_empty_name() {
        assert!(validate_product("", 10.0, 0.0).is_err());
        assert!(validate_product("  ".trim(), 10.0, 0.0).is_err());
    }

    #[test]
    fn validate_product_rejects_negative_cost() {
        assert!(validate_product("Produto", -0.01, 0.0).is_err());
    }

    #[test]
    fn validate_product_rejects_loss_rate_of_one() {
        assert!(validate_product("Produto", 10.0, 1.0).is_err());
    }

    #[test]
    fn validate_product_rejects_loss_rate_above_one() {
        assert!(validate_product("Produto", 10.0, 1.5).is_err());
    }

    #[test]
    fn validate_product_rejects_negative_loss_rate() {
        assert!(validate_product("Produto", 10.0, -0.1).is_err());
    }

    #[test]
    fn validate_product_accepts_valid_data() {
        assert!(validate_product("Bolo de Chocolate", 10.0, 0.05).is_ok());
        assert!(validate_product("Farinha", 0.0, 0.0).is_ok());
    }

    #[test]
    fn create_product_inserts_record() {
        let conn = test_db();
        insert_product(&conn, "Bolo de Cenoura", true);
        let count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Products" WHERE name = 'Bolo de Cenoura'"#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn create_product_stores_sellable_flag() {
        let conn = test_db();
        insert_product(&conn, "Farinha de Trigo", false);
        let sellable: bool = conn
            .query_row(
                r#"SELECT sellable FROM "Products" WHERE name = 'Farinha de Trigo'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!sellable);
    }

    #[test]
    fn create_product_stores_unit() {
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Products"
               (name, active, sellable, markup, cost_price, final_price, expected_loss_rate,
                unit, updated_at, created_at)
               VALUES ('Papel Rolo', true, false, 1.5, 10.0, 15.0, 0.0,
                       'rolo', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        let unit: String = conn
            .query_row(
                r#"SELECT unit FROM "Products" WHERE name = 'Papel Rolo'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unit, "rolo");
    }

    #[test]
    fn create_product_defaults_unit_to_unidade() {
        let conn = test_db();
        insert_product(&conn, "Produto Sem Unidade", true);
        let unit: String = conn
            .query_row(
                r#"SELECT unit FROM "Products" WHERE name = 'Produto Sem Unidade'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unit, "unidade");
    }

    #[test]
    fn list_products_includes_unit_field() {
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Products"
               (name, active, sellable, markup, cost_price, final_price, expected_loss_rate,
                unit, updated_at, created_at)
               VALUES ('Bobina', true, false, 1.0, 50.0, 50.0, 0.0,
                       'bobina', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        let unit: String = conn
            .query_row(
                r#"SELECT unit FROM "Products" WHERE name = 'Bobina'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unit, "bobina");
    }

    #[test]
    fn update_product_returns_zero_rows_for_missing_id() {
        let conn = test_db();
        let rows = conn
            .execute(
                r#"UPDATE "Products" SET name = 'X', updated_at = CURRENT_TIMESTAMP WHERE id = 9999"#,
                [],
            )
            .unwrap();
        assert_eq!(rows, 0);
    }

    #[test]
    fn update_product_modifies_existing_record() {
        let conn = test_db();
        let id = insert_product(&conn, "Produto Antigo", true);
        conn.execute(
            r#"UPDATE "Products" SET name = 'Produto Novo', updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![id],
        )
        .unwrap();
        let name: String = conn
            .query_row(
                r#"SELECT name FROM "Products" WHERE id = ?"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(name, "Produto Novo");
    }

    #[test]
    fn update_product_persists_unit() {
        let conn = test_db();
        let id = insert_product(&conn, "Insumo", false);
        conn.execute(
            r#"UPDATE "Products" SET unit = 'ml', updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![id],
        )
        .unwrap();
        let unit: String = conn
            .query_row(
                r#"SELECT unit FROM "Products" WHERE id = ?"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unit, "ml");
    }

    #[test]
    fn pricing_rates_no_sales_returns_zeroed_cv_df() {
        let conn = test_db();
        insert_business_settings(&conn, "Empresa");
        let faturamento: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(total_value), 0.0) FROM "Sales"
                   WHERE created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(faturamento, 0.0);
    }

    #[test]
    fn pricing_rates_calculates_cv_df_from_entries() {
        let conn = test_db();
        insert_business_settings(&conn, "Empresa");
        conn.execute(
            r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
               VALUES ('u', 'u', 'h', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();
        let user_id: i64 = conn.query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0)).unwrap();
        conn.execute(
            r#"INSERT INTO "Sales" (user_id, status, total_value, updated_at, created_at)
               VALUES (?, 'closed', 1000.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![user_id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Financial_Entries"
               (type, name, category, amount, is_fixed, updated_at, created_at)
               VALUES ('expense', 'Taxa cartão', 'variavel', 100.0, false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Financial_Entries"
               (type, name, category, amount, is_fixed, updated_at, created_at)
               VALUES ('expense', 'Aluguel', 'fixo', 200.0, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();
        let cv: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(amount), 0.0) FROM "Financial_Entries" WHERE is_fixed = false"#,
                [], |r| r.get(0),
            ).unwrap();
        let df: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(amount), 0.0) FROM "Financial_Entries"
                   WHERE "type" = 'expense' AND is_fixed = true"#,
                [], |r| r.get(0),
            ).unwrap();
        let fat: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(total_value), 0.0) FROM "Sales"
                   WHERE created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
                [], |r| r.get(0),
            ).unwrap();
        assert!((cv / fat - 0.1).abs() < 1e-9);
        assert!((df / fat - 0.2).abs() < 1e-9);
    }

    #[test]
    fn loss_rate_zero_without_movements() {
        let conn = test_db();
        let id = insert_product(&conn, "Produto", true);
        let total_in: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
                   WHERE product_id = ? AND movement_type = 'in'"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(total_in, 0.0);
    }

    #[test]
    fn loss_rate_calculated_from_movements() {
        let conn = test_db();
        let id = insert_product(&conn, "Produto", true);
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 100.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 10.0, 'out', 'perda', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        let total_in: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
                   WHERE product_id = ? AND movement_type = 'in'"#,
                params![id], |r| r.get(0),
            ).unwrap();
        let total_loss: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
                   WHERE product_id = ? AND movement_type = 'out' AND motive = 'perda'"#,
                params![id], |r| r.get(0),
            ).unwrap();
        assert!((total_loss / total_in - 0.1).abs() < 1e-9);
    }

    #[test]
    fn realized_margin_returns_zero_without_closed_sales() {
        let conn = test_db();
        let margin: f64 = conn
            .query_row(
                r#"SELECT COALESCE(
                    AVG(
                        (sp.discounted_price - p.cost_price / (1.0 - p.expected_loss_rate))
                        / NULLIF(sp.discounted_price, 0)
                    ), 0.0
                )
                FROM "Sales_Products" sp
                JOIN "Products" p ON p.id = sp.product_id
                JOIN "Sales" s ON s.id = sp.sale_id
                WHERE s.status = 'closed'
                AND s.created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
                [],
                |r| r.get(0),
            )
            .unwrap_or(0.0);
        assert_eq!(margin, 0.0);
    }

    #[test]
    fn realized_margin_calculates_from_closed_sales() {
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
               VALUES ('u', 'u', 'h', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        ).unwrap();
        let user_id: i64 = conn.query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0)).unwrap();
        let product_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Products"
                   (name, active, sellable, markup, cost_price, final_price, expected_loss_rate,
                    updated_at, created_at)
                   VALUES ('Foto', true, true, 2.0, 10.0, 20.0, 0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
                   VALUES (?, '', 'closed', 20.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products"
               (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 20.0, 0.0, 20.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();
        let margin: f64 = conn
            .query_row(
                r#"SELECT COALESCE(
                    AVG(
                        (sp.discounted_price - p.cost_price / (1.0 - p.expected_loss_rate))
                        / NULLIF(sp.discounted_price, 0)
                    ), 0.0
                )
                FROM "Sales_Products" sp
                JOIN "Products" p ON p.id = sp.product_id
                JOIN "Sales" s ON s.id = sp.sale_id
                WHERE s.status = 'closed'
                AND s.created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!((margin - 0.5).abs() < 1e-9);
    }
}
