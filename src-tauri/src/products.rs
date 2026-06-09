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
    pub loss_mode: String,
}

pub fn validate_product(name: &str, expected_loss_rate: f64) -> Result<(), String> {
    if name.is_empty() {
        return Err("Nome é obrigatório".to_string());
    }
    if !(0.0..1.0).contains(&expected_loss_rate) {
        return Err("Taxa de perda deve estar entre 0% e 99%".to_string());
    }
    Ok(())
}

pub fn validate_loss_mode(loss_mode: &str) -> Result<(), String> {
    if loss_mode != "manual" && loss_mode != "auto" {
        return Err("Modo de taxa de perda inválido".to_string());
    }
    Ok(())
}

pub(crate) fn validate_unit(
    conn: &Connection,
    unit: &str,
    require_active: bool,
) -> Result<(), String> {
    let count: i64 = if require_active {
        conn.query_row(
            r#"SELECT COUNT(*) FROM "Units_Of_Measure" WHERE name = ? AND active = true"#,
            params![unit],
            |r| r.get(0),
        )
    } else {
        conn.query_row(
            r#"SELECT COUNT(*) FROM "Units_Of_Measure" WHERE name = ?"#,
            params![unit],
            |r| r.get(0),
        )
    }
    .map_err(|e| e.to_string())?;

    if count == 0 {
        return Err(if require_active {
            format!("Unidade '{}' não existe ou está inativa", unit)
        } else {
            format!("Unidade '{}' não existe", unit)
        });
    }
    Ok(())
}

pub fn get_pricing_rates_raw(conn: &Connection) -> PricingRates {
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
        return PricingRates {
            cv: 0.0,
            df: 0.0,
            investment_rate,
            profit_margin,
            no_sales_history: true,
        };
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

    let card_fee_sum: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(sp.amount_paid - sp.net_amount), 0.0)
               FROM "Sale_Payments" sp
               WHERE sp.payment_method_id IN (
                   SELECT DISTINCT payment_method_id FROM "Transaction_Fees"
                   WHERE fee_percent > 0 OR fixed_fee > 0
               )
               AND sp.created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let card_cv: f64 = if card_fee_sum > 0.0 {
        card_fee_sum / faturamento_ref
    } else {
        conn.query_row(
            r#"SELECT COALESCE(MAX(fee_percent), 0.0) FROM "Transaction_Fees"
               WHERE fee_percent > 0"#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0)
    };

    PricingRates {
        cv: (cv_total / faturamento_ref) + card_cv,
        df: df_total / faturamento_ref,
        investment_rate,
        profit_margin,
        no_sales_history: false,
    }
}

#[tauri::command]
pub fn get_pricing_rates(state: State<DbState>) -> Result<PricingRates, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    Ok(get_pricing_rates_raw(&conn))
}

pub fn get_product_loss_rate_raw(conn: &Connection, product_id: i64) -> f64 {
    // O motivo pode vir anotado como "perda - observação" (create_stock_adjustment
    // concatena a nota). split_part extrai o motivo-base para a medição não ignorar
    // perdas registradas com observação.
    let (total_loss, total_resolved): (f64, f64) = conn
        .query_row(
            r#"SELECT
               COALESCE(SUM(CASE WHEN split_part(motive, ' - ', 1) = 'perda' THEN quantity ELSE 0 END), 0.0),
               COALESCE(SUM(quantity), 0.0)
               FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out'
               AND split_part(motive, ' - ', 1) IN ('perda', 'venda')
               AND created_at >= now()::TIMESTAMP - INTERVAL '12 MONTHS'"#,
            params![product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0.0, 0.0));

    if total_resolved == 0.0 {
        0.0
    } else {
        total_loss / total_resolved
    }
}

#[tauri::command]
pub fn get_product_loss_rate(product_id: i64, state: State<DbState>) -> Result<f64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    Ok(get_product_loss_rate_raw(&conn, product_id))
}

/// Derives CMV from Product_Components for a product with a bill of materials.
/// Returns Ok(None) if the product has no components (caller uses own cost_price).
/// Returns Err if any component has expected_loss_rate >= 1 (CMV undefined).
pub fn compute_cmv_from_components(
    conn: &Connection,
    product_id: i64,
) -> Result<Option<f64>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT pc.quantity,
                   CAST(p.cost_price AS DOUBLE),
                   CAST(p.expected_loss_rate AS DOUBLE)
            FROM "Product_Components" pc
            JOIN "Products" p ON p.id = pc.child_id
            WHERE pc.parent_id = ?
        "#,
        )
        .map_err(|e| e.to_string())?;

    let components: Vec<(i64, f64, f64)> = stmt
        .query_map(params![product_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if components.is_empty() {
        return Ok(None);
    }

    let mut cmv = 0.0_f64;
    for (quantity, cost_price, loss_rate) in &components {
        if *loss_rate >= 1.0 {
            return Err("Componente com taxa de perda de 100% impede cálculo do CMV".to_string());
        }
        cmv += (cost_price / (1.0 - loss_rate)) * (*quantity as f64);
    }
    Ok(Some(cmv))
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

    // CAST decimals to DOUBLE: duckdb-rs 1.4.4 reads DECIMAL(5,4) directly as f64 incorrectly
    // (returns 0). Casting in SQL works for both legacy DECIMAL and new DOUBLE columns.
    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, name, description, active, sellable,
                   CAST(cost_price AS DOUBLE), CAST(markup AS DOUBLE),
                   CAST(final_price AS DOUBLE), CAST(expected_loss_rate AS DOUBLE),
                   unit, CAST(profit_margin AS DOUBLE), COALESCE(loss_mode, 'manual')
            FROM "Products"
            ORDER BY name
        "#,
        )
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
                loss_mode: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(products)
}

/// Creates a product inside an already-open transaction.
/// Validates name (non-empty) and unit (active). Returns the new product id.
/// initial fields: cost_price=0, markup=0, final_price=0, profit_margin=0,
/// expected_loss_rate=0, loss_mode='manual', active=true.
pub(crate) fn create_product_in_tx(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
    unit: &str,
    sellable: bool,
) -> Result<i64, String> {
    let name = name.trim();
    validate_product(name, 0.0)?;
    validate_unit(conn, unit, true)?;
    conn.query_row(
        r#"INSERT INTO "Products"
           (name, description, active, sellable, cost_price, markup, final_price,
            expected_loss_rate, unit, profit_margin, updated_at, created_at)
           VALUES (?, ?, true, ?, 0.0, 0.0, 0.0, 0.0, ?, 0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
           RETURNING id"#,
        params![name, description, sellable, unit],
        |r| r.get(0),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_product(
    name: String,
    description: Option<String>,
    active: bool,
    sellable: bool,
    markup: f64,
    final_price: f64,
    unit: String,
    profit_margin: f64,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    validate_product(&name, 0.0)?;

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = create_product_in_tx(&conn, &name, description.as_deref(), &unit, sellable)?;
    // Set the form-provided pricing fields (create_product_in_tx starts them at 0).
    conn.execute(
        r#"UPDATE "Products"
           SET active = ?, markup = ?, final_price = ?, profit_margin = ?,
               updated_at = CURRENT_TIMESTAMP
           WHERE id = ?"#,
        params![active, markup, final_price, profit_margin, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_product(
    id: i64,
    name: String,
    description: Option<String>,
    active: bool,
    sellable: bool,
    markup: f64,
    final_price: f64,
    unit: String,
    profit_margin: f64,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    // cost_price, expected_loss_rate and loss_mode are owned by the purchase event;
    // none of them appear in the SET clause to avoid overwriting purchase-frozen values.
    validate_product(&name, 0.0)?;

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    validate_unit(&conn, &unit, false)?;

    let rows_affected = conn
        .execute(
            r#"UPDATE "Products"
               SET name = ?, description = ?, active = ?, sellable = ?,
                   markup = ?, final_price = ?,
                   unit = ?, profit_margin = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![
                name,
                description,
                active,
                sellable,
                markup,
                final_price,
                unit,
                profit_margin,
                id
            ],
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
    fn validate_loss_mode_accepts_manual_and_auto() {
        assert!(validate_loss_mode("manual").is_ok());
        assert!(validate_loss_mode("auto").is_ok());
    }

    #[test]
    fn validate_loss_mode_rejects_invalid() {
        assert!(validate_loss_mode("").is_err());
        assert!(validate_loss_mode("automatico").is_err());
    }

    #[test]
    fn loss_mode_roundtrips_and_defaults_to_manual() {
        let conn = test_db();
        // loss_mode = 'auto' explícito persiste.
        conn.execute(
            r#"INSERT INTO "Products"
               (name, active, sellable, markup, cost_price, final_price, expected_loss_rate,
                profit_margin, loss_mode, updated_at, created_at)
               VALUES ('Auto', true, true, 1.5, 10.0, 15.0, 0.0,
                       0.0, 'auto', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        // Sem loss_mode informado: assume o default 'manual' da coluna.
        conn.execute(
            r#"INSERT INTO "Products"
               (name, active, sellable, markup, cost_price, final_price, expected_loss_rate,
                profit_margin, updated_at, created_at)
               VALUES ('Padrao', true, true, 1.5, 10.0, 15.0, 0.0,
                       0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();

        let auto_mode: String = conn
            .query_row(
                r#"SELECT loss_mode FROM "Products" WHERE name = 'Auto'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        let default_mode: String = conn
            .query_row(
                r#"SELECT loss_mode FROM "Products" WHERE name = 'Padrao'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(auto_mode, "auto");
        assert_eq!(default_mode, "manual");
    }

    // Task 4.6 tests

    #[test]
    fn validate_unit_rejects_nonexistent_unit() {
        let conn = test_db();
        assert!(validate_unit(&conn, "kg", true).is_err());
        assert!(validate_unit(&conn, "kg", false).is_err());
    }

    #[test]
    fn validate_unit_for_create_rejects_inactive() {
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Units_Of_Measure" (name, abbreviation, active, updated_at, created_at)
               VALUES ('kg', 'kg', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        assert!(validate_unit(&conn, "kg", true).is_err());
    }

    #[test]
    fn validate_unit_for_update_accepts_inactive() {
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Units_Of_Measure" (name, abbreviation, active, updated_at, created_at)
               VALUES ('kg', 'kg', false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        assert!(validate_unit(&conn, "kg", false).is_ok());
    }

    #[test]
    fn validate_unit_accepts_active_unit() {
        let conn = test_db();
        use crate::settings::insert_unit_of_measure;
        insert_unit_of_measure(&conn, "kg", "kg");
        assert!(validate_unit(&conn, "kg", true).is_ok());
        assert!(validate_unit(&conn, "kg", false).is_ok());
    }

    #[test]
    fn validate_product_rejects_empty_name() {
        assert!(validate_product("", 0.0).is_err());
        assert!(validate_product("  ".trim(), 0.0).is_err());
    }

    #[test]
    fn validate_product_rejects_loss_rate_of_one() {
        assert!(validate_product("Produto", 1.0).is_err());
    }

    #[test]
    fn validate_product_rejects_loss_rate_above_one() {
        assert!(validate_product("Produto", 1.5).is_err());
    }

    #[test]
    fn validate_product_rejects_negative_loss_rate() {
        assert!(validate_product("Produto", -0.1).is_err());
    }

    #[test]
    fn validate_product_accepts_valid_data() {
        assert!(validate_product("Bolo de Chocolate", 0.05).is_ok());
        assert!(validate_product("Farinha", 0.0).is_ok());
    }

    #[test]
    fn create_product_inserts_cost_price_zero() {
        // Scenario "Cadastro de insumo sem custo manual": cost_price nasce em 0
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Products"
               (name, description, active, sellable, cost_price, markup, final_price,
                expected_loss_rate, unit, profit_margin, updated_at, created_at)
               VALUES ('Farinha', NULL, true, false, 0.0, 1.5, 0.0, 0.0, 'unidade', 0.0,
                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        let cost: f64 = conn
            .query_row(
                r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE name = 'Farinha'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cost, 0.0, "new product must start with cost_price = 0.0");
    }

    #[test]
    fn update_product_does_not_touch_cost_price() {
        // Scenario "Edição não altera o custo fixado": update preserves purchase-set cost
        let conn = test_db();
        let id = insert_product(&conn, "Insumo CP", false);
        conn.execute(
            r#"UPDATE "Products" SET cost_price = 5.50, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![id],
        )
        .unwrap();
        // Mirror of update_product SET clause (cost_price absent)
        conn.execute(
            r#"UPDATE "Products"
               SET name = ?, description = ?, active = ?, sellable = ?,
                   markup = ?, final_price = ?,
                   unit = ?, profit_margin = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![
                "Insumo CP Novo",
                Option::<String>::None,
                true,
                false,
                1.5_f64,
                8.25_f64,
                "unidade",
                0.3_f64,
                id
            ],
        )
        .unwrap();
        let cost: f64 = conn
            .query_row(
                r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            (cost - 5.50).abs() < 1e-9,
            "update_product must not modify cost_price, got {}",
            cost
        );
    }

    #[test]
    fn create_product_inserts_record() {
        let conn = test_db();
        insert_product(&conn, "Bolo de Cenoura", true);
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Products" WHERE name = 'Bolo de Cenoura'"#,
                [],
                |r| r.get(0),
            )
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
    fn product_roundtrips_fractional_profit_margin_and_loss_rate() {
        // Regression: duckdb-rs 1.4.4 reads DECIMAL(5,4) as f64 without applying scale,
        // returning 0 / wrong values. DOUBLE columns round-trip correctly.
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Products"
               (name, active, sellable, markup, cost_price, final_price, expected_loss_rate,
                profit_margin, updated_at, created_at)
               VALUES ('Margem', true, true, 1.25, 10.0, 12.5, 0.0732,
                       0.3456, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        let (margin, loss, markup): (f64, f64, f64) = conn
            .query_row(
                r#"SELECT profit_margin, expected_loss_rate, markup
                   FROM "Products" WHERE name = 'Margem'"#,
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert!(
            (margin - 0.3456).abs() < 1e-9,
            "profit_margin got {}",
            margin
        );
        assert!(
            (loss - 0.0732).abs() < 1e-9,
            "expected_loss_rate got {}",
            loss
        );
        assert!((markup - 1.25).abs() < 1e-9, "markup got {}", markup);
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
    fn update_product_does_not_touch_loss_fields() {
        // A edição do produto não altera expected_loss_rate nem loss_mode fixados pela compra.
        let conn = test_db();
        let id = insert_product(&conn, "Insumo", false);
        conn.execute(
            r#"UPDATE "Products" SET expected_loss_rate = 0.2, loss_mode = 'auto',
               updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![id],
        )
        .unwrap();
        // UPDATE espelhando a nova forma de update_product (sem colunas de perda).
        conn.execute(
            r#"UPDATE "Products"
               SET name = ?, description = ?, active = ?, sellable = ?,
                   cost_price = ?, markup = ?, final_price = ?,
                   unit = ?, profit_margin = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![
                "Insumo Novo",
                Option::<String>::None,
                true,
                false,
                12.0,
                1.5,
                18.0,
                "unidade",
                0.3,
                id
            ],
        )
        .unwrap();
        let (loss, mode): (f64, String) = conn
            .query_row(
                r#"SELECT CAST(expected_loss_rate AS DOUBLE), loss_mode FROM "Products" WHERE id = ?"#,
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!((loss - 0.2).abs() < 1e-9, "loss mudou: {}", loss);
        assert_eq!(mode, "auto");
    }

    #[test]
    fn create_product_defaults_loss_to_zero_and_manual() {
        // INSERT espelhando a nova forma de create_product (perda 0, modo no default).
        let conn = test_db();
        conn.execute(
            r#"INSERT INTO "Products"
               (name, description, active, sellable, cost_price, markup, final_price,
                expected_loss_rate, unit, profit_margin, updated_at, created_at)
               VALUES ('Novo', NULL, true, false, 5.0, 1.5, 7.5, 0.0, 'unidade', 0.0,
                       CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        let (loss, mode): (f64, String) = conn
            .query_row(
                r#"SELECT CAST(expected_loss_rate AS DOUBLE), loss_mode FROM "Products" WHERE name = 'Novo'"#,
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(loss, 0.0);
        assert_eq!(mode, "manual");
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
        )
        .unwrap();
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales" (user_id, status, total_value, updated_at, created_at)
               VALUES (?, 'closed', 1000.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![user_id],
        )
        .unwrap();
        conn.execute(
            r#"INSERT INTO "Financial_Entries"
               (type, name, category, amount, is_fixed, updated_at, created_at)
               VALUES ('expense', 'Insumo', 'variavel', 100.0, false, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
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
                [],
                |r| r.get(0),
            )
            .unwrap();
        let fat: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(total_value), 0.0) FROM "Sales"
                   WHERE created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        // No card fees configured, so card_cv = 0
        assert!((cv / fat - 0.1).abs() < 1e-9);
        assert!((df / fat - 0.2).abs() < 1e-9);
    }

    // Task 4.4 tests

    #[test]
    fn pricing_rates_cv_includes_card_fees_from_history() {
        use crate::settings::insert_payment_method;
        let conn = test_db();
        insert_business_settings(&conn, "Empresa");
        conn.execute(
            r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
               VALUES ('u', 'u', 'h', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();

        let pm_id = insert_payment_method(&conn, "Cartão");
        // Card method with a fee
        conn.execute(
            r#"INSERT INTO "Transaction_Fees" (payment_method_id, installments, fee_percent, fixed_fee, updated_at, created_at)
               VALUES (?, 1, 0.02, 0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![pm_id],
        ).unwrap();

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, status, total_value, updated_at, created_at)
               VALUES (?, 'closed', 500.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();

        // amount_paid = 100, net_amount = 98 (2% fee retained)
        conn.execute(
            r#"INSERT INTO "Sale_Payments" (sale_id, payment_method_id, amount_paid, net_amount, created_at)
               VALUES (?, ?, 100.0, 98.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, pm_id],
        ).unwrap();

        let card_fee_sum: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(amount_paid - net_amount), 0.0)
               FROM "Sale_Payments"
               WHERE payment_method_id IN (
                   SELECT DISTINCT payment_method_id FROM "Transaction_Fees" WHERE fee_percent > 0
               )
               AND created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        let fat: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(total_value), 0.0) FROM "Sales"
               WHERE created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();

        // card_cv = 2.0 / 500.0 = 0.004
        let card_cv = card_fee_sum / fat;
        assert!(
            (card_cv - 0.004).abs() < 1e-9,
            "card_cv should be 0.004, got {}",
            card_cv
        );
    }

    #[test]
    fn pricing_rates_fallback_uses_max_fee_when_no_card_history() {
        use crate::settings::insert_payment_method;
        let conn = test_db();
        insert_business_settings(&conn, "Empresa");
        conn.execute(
            r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
               VALUES ('u', 'u', 'h', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();

        let pm_id = insert_payment_method(&conn, "Cartão");
        conn.execute(
            r#"INSERT INTO "Transaction_Fees" (payment_method_id, installments, fee_percent, fixed_fee, updated_at, created_at)
               VALUES (?, 1, 0.025, 0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![pm_id],
        ).unwrap();

        // Sales exist (faturamento > 0) but no card payments
        let pix_pm_id = insert_payment_method(&conn, "Pix");
        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, status, total_value, updated_at, created_at)
               VALUES (?, 'closed', 400.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sale_Payments" (sale_id, payment_method_id, amount_paid, net_amount, created_at)
               VALUES (?, ?, 400.0, 400.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, pix_pm_id],
        ).unwrap();

        // No card payments → fallback = MAX(fee_percent) = 0.025
        let card_fee_sum: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(amount_paid - net_amount), 0.0)
               FROM "Sale_Payments"
               WHERE payment_method_id IN (
                   SELECT DISTINCT payment_method_id FROM "Transaction_Fees" WHERE fee_percent > 0
               )
               AND created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(card_fee_sum, 0.0, "no card payments, sum should be 0");

        let max_fee: f64 = conn.query_row(
            r#"SELECT COALESCE(MAX(fee_percent), 0.0) FROM "Transaction_Fees" WHERE fee_percent > 0"#,
            [], |r| r.get(0),
        ).unwrap();
        assert!(
            (max_fee - 0.025).abs() < 1e-9,
            "fallback should be max fee 0.025"
        );
    }

    #[test]
    fn pricing_rates_no_card_no_fees_card_cv_is_zero() {
        let conn = test_db();
        insert_business_settings(&conn, "Empresa");
        conn.execute(
            r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
               VALUES ('u', 'u', 'h', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            [],
        )
        .unwrap();
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();

        // No Transaction_Fees at all (Pix/dinheiro methods)
        let pm_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Payment_Methods" (name, active, updated_at, created_at)
               VALUES ('Dinheiro', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, status, total_value, updated_at, created_at)
               VALUES (?, 'closed', 200.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sale_Payments" (sale_id, payment_method_id, amount_paid, net_amount, created_at)
               VALUES (?, ?, 200.0, 200.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, pm_id],
        ).unwrap();

        let card_fee_sum: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(amount_paid - net_amount), 0.0)
               FROM "Sale_Payments"
               WHERE payment_method_id IN (
                   SELECT DISTINCT payment_method_id FROM "Transaction_Fees" WHERE fee_percent > 0
               )
               AND created_at >= now()::TIMESTAMP - INTERVAL '1 year'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        let max_fee: f64 = conn.query_row(
            r#"SELECT COALESCE(MAX(fee_percent), 0.0) FROM "Transaction_Fees" WHERE fee_percent > 0"#,
            [], |r| r.get(0),
        ).unwrap();

        assert_eq!(card_fee_sum, 0.0);
        assert_eq!(max_fee, 0.0, "no fees configured, max_fee should be 0");
    }

    #[test]
    fn loss_rate_zero_without_resolved_movements() {
        let conn = test_db();
        let id = insert_product(&conn, "Produto", false);
        // Only an 'in' movement — no resolved units
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 100.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        assert_eq!(get_product_loss_rate_raw(&conn, id), 0.0);
    }

    #[test]
    fn loss_rate_calculated_from_venda_and_perda() {
        // Scenario: 20 sold + 5 lost → rate = 5/25 = 0.2
        let conn = test_db();
        let id = insert_product(&conn, "Papel", false);
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 20.0, 'out', 'venda', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 5.0, 'out', 'perda', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        let rate = get_product_loss_rate_raw(&conn, id);
        assert!((rate - 0.2).abs() < 1e-9, "expected 0.2, got {}", rate);
    }

    #[test]
    fn loss_rate_in_movements_do_not_dilute() {
        // Scenario: 10 sold + 5 lost + 100 still in stock → rate = 5/15, not 5/115
        let conn = test_db();
        let id = insert_product(&conn, "Papel", false);
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 100.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 10.0, 'out', 'venda', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 5.0, 'out', 'perda', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        let rate = get_product_loss_rate_raw(&conn, id);
        let expected = 5.0 / 15.0;
        assert!(
            (rate - expected).abs() < 1e-9,
            "expected {}, got {}",
            expected,
            rate
        );
    }

    #[test]
    fn loss_rate_counts_annotated_perda_motive() {
        // Perda registrada com observação vira "perda - <nota>"; deve contar mesmo assim.
        // 15 vendidos + 5 perdidos (anotados) → 5 / 20 = 0.25
        let conn = test_db();
        let id = insert_product(&conn, "Papel", false);
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 15.0, 'out', 'venda', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 5.0, 'out', 'perda - Impressão com defeito', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        let rate = get_product_loss_rate_raw(&conn, id);
        assert!((rate - 0.25).abs() < 1e-9, "expected 0.25, got {}", rate);
    }

    #[test]
    fn loss_rate_ignores_movements_older_than_12_months() {
        // Movements older than 12 months must not count
        let conn = test_db();
        let id = insert_product(&conn, "Papel", false);
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 10.0, 'out', 'perda', now()::TIMESTAMP - INTERVAL '13 MONTHS')"#,
            params![id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 20.0, 'out', 'venda', now()::TIMESTAMP - INTERVAL '13 MONTHS')"#,
            params![id],
        ).unwrap();
        assert_eq!(get_product_loss_rate_raw(&conn, id), 0.0);
    }

    #[test]
    fn cmv_from_components_no_components_returns_none() {
        let conn = test_db();
        let parent = insert_product(&conn, "Foto", true);
        let result = compute_cmv_from_components(&conn, parent).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn cmv_from_components_rollup_is_correct() {
        // child cost=10, loss=0.2 → CMV per unit = 10/0.8 = 12.5; qty=2 → 25.0
        let conn = test_db();
        let parent = insert_product(&conn, "Foto", true);
        let child = insert_product(&conn, "Papel", false);
        conn.execute(
            r#"UPDATE "Products" SET cost_price = 10.0, expected_loss_rate = 0.2,
               updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![child],
        )
        .unwrap();
        conn.execute(
            r#"INSERT INTO "Product_Components" (parent_id, child_id, quantity, updated_at, created_at)
               VALUES (?, ?, 2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![parent, child],
        ).unwrap();
        let cmv = compute_cmv_from_components(&conn, parent).unwrap().unwrap();
        assert!((cmv - 25.0).abs() < 1e-9, "expected 25.0, got {}", cmv);
    }

    #[test]
    fn cmv_from_components_loss_rate_one_returns_err() {
        let conn = test_db();
        let parent = insert_product(&conn, "Foto", true);
        let child = insert_product(&conn, "Papel", false);
        conn.execute(
            r#"UPDATE "Products" SET expected_loss_rate = 1.0,
               updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![child],
        )
        .unwrap();
        conn.execute(
            r#"INSERT INTO "Product_Components" (parent_id, child_id, quantity, updated_at, created_at)
               VALUES (?, ?, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![parent, child],
        ).unwrap();
        let result = compute_cmv_from_components(&conn, parent);
        assert!(
            result.is_err(),
            "should error when component loss_rate >= 1"
        );
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
        )
        .unwrap();
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
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
        )
        .unwrap();
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
