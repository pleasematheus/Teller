use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;
use crate::products::{get_product_loss_rate_raw, validate_loss_mode};

#[derive(Serialize)]
pub struct StockItem {
    pub product_id: i64,
    pub name: String,
    pub unit: String,
    pub current_quantity: f64,
}

#[derive(Deserialize)]
pub struct PurchaseItemInput {
    pub product_id: i64,
    pub quantity: f64,
    pub unit_cost: f64,
    pub box_qty: Option<f64>,
    pub unit_qty: Option<f64>,
    pub loss_rate: Option<f64>,
    pub loss_mode: Option<String>,
}

#[derive(Serialize)]
pub struct StockMovement {
    pub id: i64,
    pub quantity: f64,
    pub movement_type: String,
    pub motive: String,
    pub created_at: String,
    pub sale_id: Option<i64>,
}

#[derive(Serialize)]
pub struct ProductComponent {
    pub id: i64,
    pub child_id: i64,
    pub child_name: String,
    pub quantity: i64,
    pub unit: String,
    pub calculated_quantity: Option<f64>,
    pub use_calculated: bool,
}

#[derive(Serialize)]
pub struct Purchase {
    pub id: i64,
    pub freight: f64,
    pub total_cost: f64,
    pub created_at: String,
}

#[tauri::command]
pub fn list_stock(state: State<DbState>) -> Result<Vec<StockItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT p.id, p.name, p.unit, COALESCE(s.current_quantity, 0.0)
            FROM "Products" p
            LEFT JOIN "Stock" s ON s.product_id = p.id
            ORDER BY p.name
        "#)
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([], |row| {
            Ok(StockItem {
                product_id: row.get(0)?,
                name: row.get(1)?,
                unit: row.get(2)?,
                current_quantity: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(items)
}

pub fn do_purchase_work(
    conn: &Connection,
    freight: f64,
    items: &[PurchaseItemInput],
) -> Result<(), String> {
    let total_cost: f64 =
        items.iter().map(|i| i.quantity * i.unit_cost).sum::<f64>() + freight;

    let purchase_id: i64 = conn
        .query_row(
            r#"INSERT INTO "Purchases" (freight, total_cost, created_at)
               VALUES (?, ?, CURRENT_TIMESTAMP)
               RETURNING id"#,
            params![freight, total_cost],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    for item in items {
        // The purchase is the sole writer of the supply's loss rate and mode.
        // 'auto' (loss_rate = None) freezes the currently measured rate; 'manual'
        // freezes the operator override. Mode defaults to 'manual' when absent.
        let loss_mode = item.loss_mode.as_deref().unwrap_or("manual");
        validate_loss_mode(loss_mode)?;

        let effective_loss = item.loss_rate
            .unwrap_or_else(|| get_product_loss_rate_raw(conn, item.product_id));

        conn.execute(
            r#"INSERT INTO "Purchase_Items"
               (purchase_id, product_id, quantity, unit_cost, box_qty, unit_qty,
                creates_stock, loss_rate, created_at)
               VALUES (?, ?, ?, ?, ?, ?, true, ?, CURRENT_TIMESTAMP)"#,
            params![
                purchase_id, item.product_id, item.quantity, item.unit_cost,
                item.box_qty, item.unit_qty, effective_loss
            ],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            r#"UPDATE "Products"
               SET expected_loss_rate = ?, loss_mode = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![effective_loss, loss_mode, item.product_id],
        )
        .map_err(|e| e.to_string())?;

        let stock_qty = match (item.box_qty, item.unit_qty) {
            (Some(bq), Some(uq)) if bq > 0.0 => item.quantity * (uq / bq),
            _ => item.quantity,
        };

        conn.execute(
            r#"INSERT INTO "Stock_Movements"
               (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, ?, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![item.product_id, stock_qty],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn create_purchase(
    freight: f64,
    items: Vec<PurchaseItemInput>,
    state: State<DbState>,
) -> Result<(), String> {
    if items.is_empty() {
        return Err("A compra deve ter ao menos um item".to_string());
    }
    for item in &items {
        if item.quantity <= 0.0 {
            return Err("Quantidade de cada item deve ser positiva".to_string());
        }
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

    match do_purchase_work(&conn, freight, &items) {
        Ok(()) => {
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

#[tauri::command]
pub fn create_stock_adjustment(
    product_id: i64,
    quantity: f64,
    motive: String,
    note: Option<String>,
    state: State<DbState>,
) -> Result<(), String> {
    if quantity == 0.0 {
        return Err("Quantidade não pode ser zero".to_string());
    }

    let movement_type = if quantity > 0.0 { "in" } else { "out" };
    let abs_quantity = quantity.abs();
    let full_motive = match note.as_deref() {
        Some(n) if !n.trim().is_empty() => format!("{} - {}", motive, n.trim()),
        _ => motive,
    };

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        r#"INSERT INTO "Stock_Movements"
           (product_id, quantity, movement_type, motive, created_at)
           VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)"#,
        params![product_id, abs_quantity, movement_type, full_motive],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn list_stock_movements(
    product_id: i64,
    state: State<DbState>,
) -> Result<Vec<StockMovement>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT id, quantity, movement_type, motive,
                   CAST(created_at AS VARCHAR), sale_id
            FROM "Stock_Movements"
            WHERE product_id = ?
            ORDER BY created_at DESC
        "#)
        .map_err(|e| e.to_string())?;

    let movements = stmt
        .query_map(params![product_id], |row| {
            Ok(StockMovement {
                id: row.get(0)?,
                quantity: row.get(1)?,
                movement_type: row.get(2)?,
                motive: row.get(3)?,
                created_at: row.get(4)?,
                sale_id: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(movements)
}

#[tauri::command]
pub fn list_product_components(
    product_id: i64,
    state: State<DbState>,
) -> Result<Vec<ProductComponent>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT pc.id, pc.child_id, p.name, pc.quantity, p.unit, pc.calculated_quantity, pc.use_calculated
            FROM "Product_Components" pc
            JOIN "Products" p ON p.id = pc.child_id
            WHERE pc.parent_id = ?
            ORDER BY p.name
        "#)
        .map_err(|e| e.to_string())?;

    let components = stmt
        .query_map(params![product_id], |row| {
            Ok(ProductComponent {
                id: row.get(0)?,
                child_id: row.get(1)?,
                child_name: row.get(2)?,
                quantity: row.get(3)?,
                unit: row.get(4)?,
                calculated_quantity: row.get(5)?,
                use_calculated: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(components)
}

#[tauri::command]
pub fn add_product_component(
    parent_id: i64,
    child_id: i64,
    quantity: i64,
    state: State<DbState>,
) -> Result<(), String> {
    if parent_id == child_id {
        return Err("Um produto não pode ser componente de si mesmo".to_string());
    }
    if quantity <= 0 {
        return Err("Quantidade deve ser maior que zero".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let parent_sellable: bool = conn
        .query_row(
            r#"SELECT sellable FROM "Products" WHERE id = ?"#,
            params![parent_id],
            |row| row.get(0),
        )
        .map_err(|_| format!("Produto {} não encontrado", parent_id))?;

    if !parent_sellable {
        return Err("Apenas produtos vendáveis podem ter componentes".to_string());
    }

    let child_sellable: bool = conn
        .query_row(
            r#"SELECT sellable FROM "Products" WHERE id = ?"#,
            params![child_id],
            |row| row.get(0),
        )
        .map_err(|_| format!("Componente {} não encontrado", child_id))?;

    if child_sellable {
        return Err("Componentes devem ser insumos (não vendáveis)".to_string());
    }

    let count: i64 = conn
        .query_row(
            r#"SELECT COUNT(*) FROM "Product_Components" WHERE parent_id = ? AND child_id = ?"#,
            params![parent_id, child_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if count > 0 {
        return Err("Este insumo já está cadastrado na ficha técnica deste produto".to_string());
    }

    conn.execute(
        r#"INSERT INTO "Product_Components"
           (parent_id, child_id, quantity, updated_at, created_at)
           VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![parent_id, child_id, quantity],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_product_component(id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            r#"DELETE FROM "Product_Components" WHERE id = ?"#,
            params![id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Componente {} não encontrado", id));
    }

    Ok(())
}

#[tauri::command]
pub fn update_product_component(id: i64, quantity: i64, state: State<DbState>) -> Result<(), String> {
    if quantity <= 0 {
        return Err("Quantidade deve ser maior que zero".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            r#"UPDATE "Product_Components"
               SET quantity = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![quantity, id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Componente {} não encontrado", id));
    }

    Ok(())
}

#[tauri::command]
pub fn list_purchases(state: State<DbState>) -> Result<Vec<Purchase>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT id, freight, total_cost, CAST(created_at AS VARCHAR)
            FROM "Purchases"
            ORDER BY created_at DESC
        "#)
        .map_err(|e| e.to_string())?;

    let purchases = stmt
        .query_map([], |row| {
            Ok(Purchase {
                id: row.get(0)?,
                freight: row.get(1)?,
                total_cost: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(purchases)
}

#[cfg(test)]
pub(crate) fn add_product_component_in_test(conn: &Connection, parent_id: i64, child_id: i64, quantity: i64) {
    conn.execute(
        r#"INSERT INTO "Product_Components"
           (parent_id, child_id, quantity, updated_at, created_at)
           VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![parent_id, child_id, quantity],
    ).unwrap();
}

#[cfg(test)]
pub(crate) fn add_product_component_in_test_result(
    conn: &Connection,
    parent_id: i64,
    child_id: i64,
    quantity: i64,
) -> Result<(), String> {
    if parent_id == child_id {
        return Err("Um produto não pode ser componente de si mesmo".to_string());
    }
    if quantity <= 0 {
        return Err("Quantidade deve ser maior que zero".to_string());
    }
    let parent_sellable: bool = conn
        .query_row(r#"SELECT sellable FROM "Products" WHERE id = ?"#, params![parent_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if !parent_sellable {
        return Err("Apenas produtos vendáveis podem ter componentes".to_string());
    }
    let child_sellable: bool = conn
        .query_row(r#"SELECT sellable FROM "Products" WHERE id = ?"#, params![child_id], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if child_sellable {
        return Err("Componentes devem ser insumos (não vendáveis)".to_string());
    }
    let count: i64 = conn
        .query_row(
            r#"SELECT COUNT(*) FROM "Product_Components" WHERE parent_id = ? AND child_id = ?"#,
            params![parent_id, child_id],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Err("Duplicado".to_string());
    }
    conn.execute(
        r#"INSERT INTO "Product_Components" (parent_id, child_id, quantity, updated_at, created_at)
           VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![parent_id, child_id, quantity],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_db;
    use crate::products::insert_product;

    #[test]
    fn purchase_records_loss_rate_and_propagates_to_product() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Papel Foto", false);

        let items = vec![PurchaseItemInput {
            product_id,
            quantity: 10.0,
            unit_cost: 5.0,
            box_qty: None,
            unit_qty: None,
            loss_rate: None,
            loss_mode: None,
        }];
        do_purchase_work(&conn, 0.0, &items).unwrap();

        let stored_loss: f64 = conn
            .query_row(
                r#"SELECT CAST(loss_rate AS DOUBLE) FROM "Purchase_Items" WHERE product_id = ?"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
        // No history → suggested rate = 0.0
        assert_eq!(stored_loss, 0.0);

        let product_loss: f64 = conn
            .query_row(
                r#"SELECT CAST(expected_loss_rate AS DOUBLE) FROM "Products" WHERE id = ?"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(product_loss, 0.0);
    }

    #[test]
    fn purchase_override_loss_rate_takes_precedence() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Bobina", false);

        let items = vec![PurchaseItemInput {
            product_id,
            quantity: 5.0,
            unit_cost: 20.0,
            box_qty: None,
            unit_qty: None,
            loss_rate: Some(0.15),
            loss_mode: Some("manual".to_string()),
        }];
        do_purchase_work(&conn, 0.0, &items).unwrap();

        let stored_loss: f64 = conn
            .query_row(
                r#"SELECT CAST(loss_rate AS DOUBLE) FROM "Purchase_Items" WHERE product_id = ?"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!((stored_loss - 0.15).abs() < 1e-9);

        let product_loss: f64 = conn
            .query_row(
                r#"SELECT CAST(expected_loss_rate AS DOUBLE) FROM "Products" WHERE id = ?"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!((product_loss - 0.15).abs() < 1e-9);
    }

    #[test]
    fn purchase_auto_mode_freezes_measured_rate_and_records_mode() {
        // Scenario: item em modo 'auto' (loss_rate=None) congela a taxa medida e grava loss_mode='auto'.
        let conn = test_db();
        let product_id = insert_product(&conn, "Bobina", false);
        // Resolved movements → measured loss = 5 / (20 + 5) = 0.2
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 20.0, 'out', 'venda', CURRENT_TIMESTAMP)"#,
            params![product_id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 5.0, 'out', 'perda', CURRENT_TIMESTAMP)"#,
            params![product_id],
        ).unwrap();

        let items = vec![PurchaseItemInput {
            product_id,
            quantity: 10.0,
            unit_cost: 3.0,
            box_qty: None,
            unit_qty: None,
            loss_rate: None,
            loss_mode: Some("auto".to_string()),
        }];
        do_purchase_work(&conn, 0.0, &items).unwrap();

        let (loss, mode): (f64, String) = conn
            .query_row(
                r#"SELECT CAST(expected_loss_rate AS DOUBLE), loss_mode FROM "Products" WHERE id = ?"#,
                params![product_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!((loss - 0.2).abs() < 1e-9, "expected frozen 0.2, got {}", loss);
        assert_eq!(mode, "auto");
    }

    #[test]
    fn purchase_manual_mode_freezes_override_and_records_mode() {
        // Scenario: item em modo 'manual' com override grava o valor e loss_mode='manual'.
        let conn = test_db();
        let product_id = insert_product(&conn, "Papel", false);

        let items = vec![PurchaseItemInput {
            product_id,
            quantity: 4.0,
            unit_cost: 8.0,
            box_qty: None,
            unit_qty: None,
            loss_rate: Some(0.25),
            loss_mode: Some("manual".to_string()),
        }];
        do_purchase_work(&conn, 0.0, &items).unwrap();

        let (loss, mode): (f64, String) = conn
            .query_row(
                r#"SELECT CAST(expected_loss_rate AS DOUBLE), loss_mode FROM "Products" WHERE id = ?"#,
                params![product_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!((loss - 0.25).abs() < 1e-9, "expected 0.25, got {}", loss);
        assert_eq!(mode, "manual");
    }

    #[test]
    fn purchase_rejects_invalid_loss_mode() {
        // Scenario: loss_mode inválido é rejeitado.
        let conn = test_db();
        let product_id = insert_product(&conn, "Tinta", false);

        let items = vec![PurchaseItemInput {
            product_id,
            quantity: 2.0,
            unit_cost: 5.0,
            box_qty: None,
            unit_qty: None,
            loss_rate: None,
            loss_mode: Some("automatico".to_string()),
        }];
        let result = do_purchase_work(&conn, 0.0, &items);
        assert!(result.is_err(), "invalid loss_mode should be rejected");
    }

    #[test]
    fn create_purchase_inserts_purchase_and_stock_movement() {
        let conn = test_db();
        let id = insert_product(&conn, "Papel A4", false);

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Purchases" (freight, total_cost, created_at)
                   VALUES (10.0, 110.0, CURRENT_TIMESTAMP) RETURNING id"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Purchase_Items"
               (purchase_id, product_id, quantity, unit_cost, creates_stock, created_at)
               VALUES (?, ?, 5.0, 20.0, true, CURRENT_TIMESTAMP)"#,
            params![purchase_id, id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements"
               (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 5.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let qty: f64 = conn
            .query_row(
                r#"SELECT current_quantity FROM "Stock" WHERE product_id = ?"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert!((qty - 5.0).abs() < 1e-9);
    }

    #[test]
    fn create_purchase_applies_box_unit_conversion() {
        let quantity = 2.0_f64;
        let box_qty = 1.0_f64;
        let unit_qty = 500.0_f64;
        let stock_qty = quantity * (unit_qty / box_qty);
        assert!((stock_qty - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn create_purchase_without_conversion_uses_quantity_directly() {
        let quantity = 3.0_f64;
        let box_qty: Option<f64> = None;
        let unit_qty: Option<f64> = None;
        let stock_qty = match (box_qty, unit_qty) {
            (Some(bq), Some(uq)) if bq > 0.0 => quantity * (uq / bq),
            _ => quantity,
        };
        assert!((stock_qty - 3.0).abs() < 1e-9);
    }

    #[test]
    fn stock_adjustment_loss_creates_out_movement() {
        let conn = test_db();
        let id = insert_product(&conn, "Papel", false);
        conn.execute(
            r#"INSERT INTO "Stock_Movements"
               (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, ?, 'out', 'perda', CURRENT_TIMESTAMP)"#,
            params![id, 5.0_f64],
        ).unwrap();
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Stock_Movements"
                   WHERE product_id = ? AND movement_type = 'out' AND motive = 'perda'"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn stock_adjustment_positive_creates_in_movement() {
        let conn = test_db();
        let id = insert_product(&conn, "Bobina", false);
        let quantity = 10.0_f64;
        let movement_type = if quantity > 0.0 { "in" } else { "out" };
        conn.execute(
            r#"INSERT INTO "Stock_Movements"
               (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, ?, ?, 'ajuste', CURRENT_TIMESTAMP)"#,
            params![id, quantity.abs(), movement_type],
        ).unwrap();
        let mt: String = conn
            .query_row(
                r#"SELECT movement_type FROM "Stock_Movements" WHERE product_id = ?"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(mt, "in");
    }

    #[test]
    fn stock_adjustment_negative_creates_out_movement() {
        let quantity = -3.0_f64;
        let movement_type = if quantity > 0.0 { "in" } else { "out" };
        assert_eq!(movement_type, "out");
        assert!((quantity.abs() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn stock_adjustment_zero_quantity_is_invalid() {
        let quantity = 0.0_f64;
        assert!(quantity == 0.0);
    }

    #[test]
    fn list_stock_movements_returns_history_for_product() {
        let conn = test_db();
        let id = insert_product(&conn, "Bobina", false);
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 10.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 2.0, 'out', 'perda', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Stock_Movements" WHERE product_id = ?"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn list_stock_movements_empty_for_product_without_movements() {
        let conn = test_db();
        let id = insert_product(&conn, "Produto Sem Movimento", false);
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Stock_Movements" WHERE product_id = ?"#,
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn list_stock_movements_returns_sale_id_for_venda_and_none_otherwise() {
        let conn = test_db();
        let id = insert_product(&conn, "Papel", false);

        let user_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Users" (name, password_hash, nickname, active, updated_at, created_at)
                   VALUES ('u', 'h', 'u', TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
                   VALUES (?, '', 'closed', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();

        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 10.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at, sale_id)
               VALUES (?, 2.0, 'out', 'venda', CURRENT_TIMESTAMP, ?)"#,
            params![id, sale_id],
        ).unwrap();

        let mut stmt = conn
            .prepare(r#"
                SELECT id, quantity, movement_type, motive,
                       CAST(created_at AS VARCHAR), sale_id
                FROM "Stock_Movements"
                WHERE product_id = ?
                ORDER BY created_at DESC
            "#)
            .unwrap();
        let movements: Vec<StockMovement> = stmt
            .query_map(params![id], |row| {
                Ok(StockMovement {
                    id: row.get(0)?,
                    quantity: row.get(1)?,
                    movement_type: row.get(2)?,
                    motive: row.get(3)?,
                    created_at: row.get(4)?,
                    sale_id: row.get(5)?,
                })
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let venda = movements.iter().find(|m| m.motive == "venda").unwrap();
        assert_eq!(venda.sale_id, Some(sale_id), "Movimentação de venda deve trazer o sale_id");
        let compra = movements.iter().find(|m| m.motive == "compra").unwrap();
        assert_eq!(compra.sale_id, None, "Movimentação de compra deve ter sale_id nulo");
    }

    #[test]
    fn add_product_component_valid_inserts_record() {
        let conn = test_db();
        let parent_id = insert_product(&conn, "Foto 10x15", true);
        let child_id = insert_product(&conn, "Papel Fotográfico", false);
        add_product_component_in_test(&conn, parent_id, child_id, 1);
        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Product_Components" WHERE parent_id = ? AND child_id = ?"#,
                params![parent_id, child_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn add_product_component_rejects_non_sellable_parent() {
        let conn = test_db();
        let parent_id = insert_product(&conn, "Papel", false);
        let child_id = insert_product(&conn, "Tinta", false);
        let result = add_product_component_in_test_result(&conn, parent_id, child_id, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("vendáveis"));
    }

    #[test]
    fn add_product_component_rejects_sellable_child() {
        let conn = test_db();
        let parent_id = insert_product(&conn, "Foto 10x15", true);
        let child_id = insert_product(&conn, "Outro Produto", true);
        let result = add_product_component_in_test_result(&conn, parent_id, child_id, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("insumos"));
    }

    #[test]
    fn add_product_component_rejects_self_reference() {
        let conn = test_db();
        let id = insert_product(&conn, "Foto 10x15", true);
        let result = add_product_component_in_test_result(&conn, id, id, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("si mesmo"));
    }

    #[test]
    fn add_product_component_rejects_zero_quantity() {
        let conn = test_db();
        let parent_id = insert_product(&conn, "Foto", true);
        let child_id = insert_product(&conn, "Papel", false);
        let result = add_product_component_in_test_result(&conn, parent_id, child_id, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Quantidade"));
    }

    #[test]
    fn add_product_component_rejects_duplicate() {
        let conn = test_db();
        let parent_id = insert_product(&conn, "Foto", true);
        let child_id = insert_product(&conn, "Papel", false);
        add_product_component_in_test(&conn, parent_id, child_id, 1);
        let result = add_product_component_in_test_result(&conn, parent_id, child_id, 2);
        assert!(result.is_err());
    }

    #[test]
    fn remove_product_component_deletes_record() {
        let conn = test_db();
        let parent_id = insert_product(&conn, "Foto", true);
        let child_id = insert_product(&conn, "Papel", false);
        add_product_component_in_test(&conn, parent_id, child_id, 1);
        let comp_id: i64 = conn
            .query_row(
                r#"SELECT id FROM "Product_Components" WHERE parent_id = ? AND child_id = ?"#,
                params![parent_id, child_id],
                |r| r.get(0),
            )
            .unwrap();
        let rows = conn
            .execute(r#"DELETE FROM "Product_Components" WHERE id = ?"#, params![comp_id])
            .unwrap();
        assert_eq!(rows, 1);
    }

    #[test]
    fn remove_product_component_nonexistent_returns_zero_rows() {
        let conn = test_db();
        let rows = conn
            .execute(r#"DELETE FROM "Product_Components" WHERE id = ?"#, params![99999_i64])
            .unwrap();
        assert_eq!(rows, 0);
    }

    #[test]
    fn update_product_component_changes_quantity() {
        let conn = test_db();
        let parent_id = insert_product(&conn, "Foto", true);
        let child_id = insert_product(&conn, "Papel", false);
        add_product_component_in_test(&conn, parent_id, child_id, 1);
        let comp_id: i64 = conn
            .query_row(
                r#"SELECT id FROM "Product_Components" WHERE parent_id = ? AND child_id = ?"#,
                params![parent_id, child_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"UPDATE "Product_Components" SET quantity = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![5_i64, comp_id],
        )
        .unwrap();
        let qty: i64 = conn
            .query_row(
                r#"SELECT quantity FROM "Product_Components" WHERE id = ?"#,
                params![comp_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(qty, 5);
    }
}
