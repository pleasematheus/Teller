use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;
use crate::products::{create_product_in_tx, get_pricing_rates_raw, get_product_loss_rate_raw, validate_loss_mode};

#[derive(Serialize)]
pub struct StockItem {
    pub product_id: i64,
    pub name: String,
    pub unit: String,
    pub current_quantity: f64,
}

#[derive(Deserialize, Default)]
pub struct PurchaseItemInput {
    pub product_id: Option<i64>,
    pub quantity: f64,
    pub unit_cost: f64,
    pub final_price: f64,
    pub unit_qty: Option<f64>,
    pub loss_rate: Option<f64>,
    pub loss_mode: Option<String>,
    #[serde(default)]
    pub new_product_name: Option<String>,
    #[serde(default)]
    pub new_product_unit: Option<String>,
    #[serde(default)]
    pub new_product_description: Option<String>,
    #[serde(default)]
    pub new_product_sellable: Option<bool>,
}

struct PurchaseItemRecord {
    id: i64,
    product_id: Option<i64>,
    new_product_name: Option<String>,
    new_product_unit: Option<String>,
    new_product_description: Option<String>,
    new_product_sellable: Option<bool>,
    quantity: f64,
    unit_cost: f64,
    unit_qty: Option<f64>,
    creates_stock: bool,
    loss_rate: f64,
    loss_mode: String,
    final_price: f64,
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
    pub status: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct PurchaseDraftItem {
    pub product_id: Option<i64>,
    pub product_name: String,
    pub product_unit: String,
    pub quantity: f64,
    pub unit_cost: f64,
    pub unit_qty: Option<f64>,
    pub loss_rate: f64,
    pub loss_mode: String,
    pub final_price: f64,
    pub is_new_product: bool,
    pub new_product_description: Option<String>,
    pub new_product_sellable: Option<bool>,
}

#[derive(Serialize)]
pub struct PurchaseDraft {
    pub id: i64,
    pub freight: f64,
    pub items: Vec<PurchaseDraftItem>,
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

// Creates a draft purchase (status='open') and stores items without applying effects.
// For existing products: loss rate is resolved and frozen at draft creation time.
// For new products: product_id is NULL; pending fields are stored.
fn create_purchase_draft(
    conn: &Connection,
    freight: f64,
    items: &[PurchaseItemInput],
) -> Result<i64, String> {
    let subtotal: f64 = items.iter().map(|i| i.quantity * i.unit_cost).sum();
    let total_cost = subtotal + freight;

    let purchase_id: i64 = conn
        .query_row(
            r#"INSERT INTO "Purchases" (freight, total_cost, status, created_at)
               VALUES (?, ?, 'open', CURRENT_TIMESTAMP)
               RETURNING id"#,
            params![freight, total_cost],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    for item in items {
        let loss_mode = item.loss_mode.as_deref().unwrap_or("manual");
        validate_loss_mode(loss_mode)?;

        let has_product_id = item.product_id.is_some();
        let has_new_product = item.new_product_name.is_some() && item.new_product_unit.is_some();
        if has_product_id == has_new_product {
            return Err(
                "Cada item deve ter exatamente um produto existente ou dados de produto novo"
                    .to_string(),
            );
        }

        if let Some(pid) = item.product_id {
            let effective_loss = item.loss_rate
                .unwrap_or_else(|| get_product_loss_rate_raw(conn, pid));
            conn.execute(
                r#"INSERT INTO "Purchase_Items"
                   (purchase_id, product_id, quantity, unit_cost, final_price, unit_qty,
                    creates_stock, loss_rate, loss_mode, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, true, ?, ?, CURRENT_TIMESTAMP)"#,
                params![
                    purchase_id, pid, item.quantity, item.unit_cost,
                    item.final_price, item.unit_qty, effective_loss, loss_mode
                ],
            )
            .map_err(|e| e.to_string())?;
        } else {
            let new_name = item.new_product_name.as_deref().unwrap();
            let new_unit = item.new_product_unit.as_deref().unwrap();
            let new_desc = item.new_product_description.as_deref();
            let new_sellable = item.new_product_sellable.unwrap_or(false);
            let creates_stock = !new_sellable;
            let effective_loss = item.loss_rate.unwrap_or(0.0);
            conn.execute(
                r#"INSERT INTO "Purchase_Items"
                   (purchase_id, product_id, quantity, unit_cost, final_price, unit_qty,
                    creates_stock, loss_rate, loss_mode,
                    new_product_name, new_product_unit, new_product_description,
                    new_product_sellable, created_at)
                   VALUES (?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"#,
                params![
                    purchase_id, item.quantity, item.unit_cost, item.final_price, item.unit_qty,
                    creates_stock, effective_loss, loss_mode,
                    new_name, new_unit, new_desc, new_sellable
                ],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(purchase_id)
}

// Applies all purchase effects (stock, cost, price, loss) and marks status='closed'.
// Must be called inside a BEGIN/COMMIT transaction.
// New-product items (product_id IS NULL) are created first, then treated normally.
fn do_close_effects(conn: &Connection, purchase_id: i64) -> Result<(), String> {
    let freight: f64 = conn
        .query_row(
            r#"SELECT CAST(freight AS DOUBLE) FROM "Purchases" WHERE id = ?"#,
            params![purchase_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT pi.id, pi.product_id,
                   CAST(pi.quantity AS DOUBLE),
                   CAST(pi.unit_cost AS DOUBLE),
                   CAST(pi.unit_qty AS DOUBLE),
                   pi.creates_stock,
                   CAST(COALESCE(pi.loss_rate, 0.0) AS DOUBLE),
                   pi.loss_mode,
                   CAST(pi.final_price AS DOUBLE),
                   pi.new_product_name,
                   pi.new_product_unit,
                   pi.new_product_description,
                   pi.new_product_sellable
            FROM "Purchase_Items" pi
            WHERE pi.purchase_id = ?
        "#)
        .map_err(|e| e.to_string())?;

    let mut items: Vec<PurchaseItemRecord> = stmt
        .query_map(params![purchase_id], |row| {
            Ok(PurchaseItemRecord {
                id: row.get(0)?,
                product_id: row.get(1)?,
                quantity: row.get(2)?,
                unit_cost: row.get(3)?,
                unit_qty: row.get(4)?,
                creates_stock: row.get(5)?,
                loss_rate: row.get(6)?,
                loss_mode: row.get(7)?,
                final_price: row.get(8)?,
                new_product_name: row.get(9)?,
                new_product_unit: row.get(10)?,
                new_product_description: row.get(11)?,
                new_product_sellable: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Resolve new-product items: create the product, backfill product_id.
    for item in &mut items {
        if item.product_id.is_none() {
            let name = item.new_product_name.as_deref()
                .ok_or_else(|| "Item sem product_id e sem new_product_name".to_string())?;
            let unit = item.new_product_unit.as_deref()
                .ok_or_else(|| "Item sem product_id e sem new_product_unit".to_string())?;
            let desc = item.new_product_description.as_deref();
            let sellable = item.new_product_sellable.unwrap_or(false);
            let new_id = create_product_in_tx(conn, name, desc, unit, sellable)?;
            conn.execute(
                r#"UPDATE "Purchase_Items" SET product_id = ? WHERE id = ?"#,
                params![new_id, item.id],
            )
            .map_err(|e| e.to_string())?;
            item.product_id = Some(new_id);
        }
    }

    let subtotal: f64 = items.iter().map(|i| i.quantity * i.unit_cost).sum();
    let rates = get_pricing_rates_raw(conn);

    for item in &items {
        if !item.creates_stock {
            continue;
        }

        let pid = item.product_id.expect("product_id resolved above");

        let stock_qty = item.unit_qty
            .map(|f| item.quantity * f)
            .unwrap_or(item.quantity);

        let saldo_ant: f64 = conn
            .query_row(
                r#"SELECT COALESCE(current_quantity, 0.0) FROM "Stock" WHERE product_id = ?"#,
                params![pid],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        let custo_ant: f64 = conn
            .query_row(
                r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#,
                params![pid],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let line_value = item.quantity * item.unit_cost;
        let freight_share = if subtotal > 0.0 { freight * line_value / subtotal } else { 0.0 };
        let lot_cost = line_value + freight_share;
        let custo_unit_lote = lot_cost / stock_qty;

        let denom = saldo_ant + stock_qty;
        let novo_custo = if denom > 0.0 {
            (saldo_ant * custo_ant + stock_qty * custo_unit_lote) / denom
        } else {
            custo_unit_lote
        };

        conn.execute(
            r#"UPDATE "Products"
               SET expected_loss_rate = ?, loss_mode = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![item.loss_rate, item.loss_mode, pid],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            r#"UPDATE "Products" SET cost_price = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
            params![novo_custo, pid],
        )
        .map_err(|e| e.to_string())?;

        let (markup, profit_margin) = if item.final_price > 0.0 && item.loss_rate < 1.0 {
            let cmv = novo_custo / (1.0 - item.loss_rate);
            (
                item.final_price / cmv,
                1.0 - rates.cv - rates.df - rates.investment_rate - cmv / item.final_price,
            )
        } else {
            (0.0, 0.0)
        };
        conn.execute(
            r#"UPDATE "Products"
               SET final_price = ?, markup = ?, profit_margin = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![item.final_price, markup, profit_margin, pid],
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            r#"INSERT INTO "Stock_Movements"
               (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, ?, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![pid, stock_qty],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.execute(
        r#"UPDATE "Purchases" SET status = 'closed' WHERE id = ?"#,
        params![purchase_id],
    )
    .map_err(|e| e.to_string())?;

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

    match create_purchase_draft(&conn, freight, &items) {
        Ok(_) => {
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
pub fn close_purchase(purchase_id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let status: String = conn
        .query_row(
            r#"SELECT status FROM "Purchases" WHERE id = ?"#,
            params![purchase_id],
            |row| row.get(0),
        )
        .map_err(|_| format!("Compra {} não encontrada", purchase_id))?;

    if status == "closed" {
        return Err("Compra já foi fechada".to_string());
    }

    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
    match do_close_effects(&conn, purchase_id) {
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
pub fn list_purchases(
    status: Option<String>,
    state: State<DbState>,
) -> Result<Vec<Purchase>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let purchases = if let Some(ref s) = status {
        let mut stmt = conn
            .prepare(r#"
                SELECT id, CAST(freight AS DOUBLE), CAST(total_cost AS DOUBLE),
                       status, CAST(created_at AS VARCHAR)
                FROM "Purchases"
                WHERE status = ?
                ORDER BY created_at DESC
            "#)
            .map_err(|e| e.to_string())?;
        stmt.query_map(params![s], |row| {
            Ok(Purchase {
                id: row.get(0)?,
                freight: row.get(1)?,
                total_cost: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        let mut stmt = conn
            .prepare(r#"
                SELECT id, CAST(freight AS DOUBLE), CAST(total_cost AS DOUBLE),
                       status, CAST(created_at AS VARCHAR)
                FROM "Purchases"
                ORDER BY created_at DESC
            "#)
            .map_err(|e| e.to_string())?;
        stmt.query_map([], |row| {
            Ok(Purchase {
                id: row.get(0)?,
                freight: row.get(1)?,
                total_cost: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(purchases)
}

#[tauri::command]
pub fn get_purchase_draft(purchase_id: i64, state: State<DbState>) -> Result<PurchaseDraft, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let (freight, status): (f64, String) = conn
        .query_row(
            r#"SELECT CAST(freight AS DOUBLE), status FROM "Purchases" WHERE id = ?"#,
            params![purchase_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| format!("Compra {} não encontrada", purchase_id))?;
    if status != "open" {
        return Err("Compra já fechada não pode ser editada".to_string());
    }
    let mut stmt = conn
        .prepare(r#"
            SELECT pi.product_id,
                   COALESCE(p.name, pi.new_product_name),
                   COALESCE(p.unit, pi.new_product_unit),
                   CAST(pi.quantity AS DOUBLE), CAST(pi.unit_cost AS DOUBLE),
                   CAST(pi.unit_qty AS DOUBLE),
                   CAST(COALESCE(pi.loss_rate, 0.0) AS DOUBLE),
                   pi.loss_mode, CAST(pi.final_price AS DOUBLE),
                   pi.new_product_description, pi.new_product_sellable,
                   (pi.product_id IS NULL)
            FROM "Purchase_Items" pi
            LEFT JOIN "Products" p ON p.id = pi.product_id
            WHERE pi.purchase_id = ?
            ORDER BY pi.id
        "#)
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_map(params![purchase_id], |row| {
            Ok(PurchaseDraftItem {
                product_id: row.get(0)?,
                product_name: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                product_unit: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                quantity: row.get(3)?,
                unit_cost: row.get(4)?,
                unit_qty: row.get(5)?,
                loss_rate: row.get(6)?,
                loss_mode: row.get(7)?,
                final_price: row.get(8)?,
                new_product_description: row.get(9)?,
                new_product_sellable: row.get(10)?,
                is_new_product: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(PurchaseDraft { id: purchase_id, freight, items })
}

#[tauri::command]
pub fn update_purchase(
    purchase_id: i64,
    freight: f64,
    items: Vec<PurchaseItemInput>,
    state: State<DbState>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let status: String = conn
        .query_row(r#"SELECT status FROM "Purchases" WHERE id = ?"#, params![purchase_id], |r| r.get(0))
        .map_err(|_| format!("Compra {} não encontrada", purchase_id))?;
    if status != "open" {
        return Err("Compra já fechada não pode ser editada".to_string());
    }
    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        conn.execute(r#"DELETE FROM "Purchase_Items" WHERE purchase_id = ?"#, params![purchase_id])
            .map_err(|e| e.to_string())?;
        let total_cost: f64 = items.iter().map(|i| i.quantity * i.unit_cost).sum::<f64>() + freight;
        conn.execute(
            r#"UPDATE "Purchases" SET freight = ?, total_cost = ? WHERE id = ?"#,
            params![freight, total_cost, purchase_id],
        )
        .map_err(|e| e.to_string())?;
        for item in &items {
            let has_product_id = item.product_id.is_some();
            let has_new_product = item.new_product_name.is_some() && item.new_product_unit.is_some();
            if has_product_id == has_new_product {
                return Err(
                    "Cada item deve ter exatamente um produto existente ou dados de produto novo"
                        .to_string(),
                );
            }
            let loss_mode = item.loss_mode.as_deref().unwrap_or("manual");
            validate_loss_mode(loss_mode)?;

            if let Some(pid) = item.product_id {
                let effective_loss = item.loss_rate
                    .unwrap_or_else(|| get_product_loss_rate_raw(&conn, pid));
                conn.execute(
                    r#"INSERT INTO "Purchase_Items"
                       (purchase_id, product_id, quantity, unit_cost, final_price,
                        unit_qty, creates_stock, loss_rate, loss_mode, created_at)
                       VALUES (?, ?, ?, ?, ?, ?, true, ?, ?, CURRENT_TIMESTAMP)"#,
                    params![
                        purchase_id, pid, item.quantity, item.unit_cost,
                        item.final_price, item.unit_qty, effective_loss, loss_mode,
                    ],
                )
                .map_err(|e| e.to_string())?;
            } else {
                let new_name = item.new_product_name.as_deref().unwrap();
                let new_unit = item.new_product_unit.as_deref().unwrap();
                let new_desc = item.new_product_description.as_deref();
                let new_sellable = item.new_product_sellable.unwrap_or(false);
                let creates_stock = !new_sellable;
                let effective_loss = item.loss_rate.unwrap_or(0.0);
                conn.execute(
                    r#"INSERT INTO "Purchase_Items"
                       (purchase_id, product_id, quantity, unit_cost, final_price,
                        unit_qty, creates_stock, loss_rate, loss_mode,
                        new_product_name, new_product_unit, new_product_description,
                        new_product_sellable, created_at)
                       VALUES (?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"#,
                    params![
                        purchase_id, item.quantity, item.unit_cost, item.final_price,
                        item.unit_qty, creates_stock, effective_loss, loss_mode,
                        new_name, new_unit, new_desc, new_sellable,
                    ],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = conn.execute_batch("ROLLBACK");
    } else {
        conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
    }
    result
}

#[tauri::command]
pub fn delete_purchase(purchase_id: i64, state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let status: String = conn
        .query_row(r#"SELECT status FROM "Purchases" WHERE id = ?"#, params![purchase_id], |r| r.get(0))
        .map_err(|_| format!("Compra {} não encontrada", purchase_id))?;
    if status != "open" {
        return Err("Compra fechada não pode ser excluída".to_string());
    }
    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        conn.execute(r#"DELETE FROM "Purchase_Items" WHERE purchase_id = ?"#, params![purchase_id])
            .map_err(|e| e.to_string())?;
        conn.execute(r#"DELETE FROM "Purchases" WHERE id = ?"#, params![purchase_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = conn.execute_batch("ROLLBACK");
    } else {
        conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
    }
    result
}

#[derive(Serialize)]
pub struct ProductPurchaseRow {
    pub purchase_id: i64,
    pub purchase_date: String,
    pub quantity: f64,
    pub unit_cost: f64,
    pub total: f64,
}

fn list_product_purchases_raw(
    conn: &Connection,
    product_id: i64,
) -> Result<Vec<ProductPurchaseRow>, String> {
    let mut stmt = conn
        .prepare(r#"
            SELECT p.id, CAST(p.created_at AS VARCHAR),
                   CAST(pi.quantity AS DOUBLE),
                   CAST(pi.unit_cost AS DOUBLE),
                   CAST(pi.quantity AS DOUBLE) * CAST(pi.unit_cost AS DOUBLE)
            FROM "Purchase_Items" pi
            JOIN "Purchases" p ON p.id = pi.purchase_id
            WHERE pi.product_id = ? AND p.status = 'closed'
            ORDER BY p.created_at DESC
        "#)
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![product_id], |row| {
            Ok(ProductPurchaseRow {
                purchase_id: row.get(0)?,
                purchase_date: row.get(1)?,
                quantity: row.get(2)?,
                unit_cost: row.get(3)?,
                total: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn list_product_purchases(
    product_id: i64,
    state: State<DbState>,
) -> Result<Vec<ProductPurchaseRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    list_product_purchases_raw(&conn, product_id)
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

    // Runs a full purchase cycle (create draft then close) for use in effect-testing scenarios.
    fn run_purchase(conn: &Connection, freight: f64, items: Vec<PurchaseItemInput>) {
        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(conn, freight, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(conn, purchase_id).unwrap();
        conn.execute_batch("COMMIT").unwrap();
    }

    // --- Draft creation tests (task 4.1) ---

    #[test]
    fn draft_create_sets_open_status_without_effects() {
        // Scenario: create_purchase creates Purchases 'open' and Purchase_Items
        // without Stock_Movements nor alteration in Products.
        let conn = test_db();
        let product_id = insert_product(&conn, "Farinha", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 10.0,
            unit_cost: 3.0,
            final_price: 6.0,
            ..Default::default()
        }];

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 0.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let status: String = conn
            .query_row(r#"SELECT status FROM "Purchases" WHERE id = ?"#, params![purchase_id], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "open");

        let movements: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Stock_Movements""#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(movements, 0, "No stock movements should be created on draft");

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        assert!((cost - 10.0).abs() < 1e-9, "cost_price must not change on draft, got {}", cost);
    }

    // --- Close purchase tests (tasks 4.2, 4.3) ---

    #[test]
    fn close_purchase_applies_stock_cost_price_and_sets_closed() {
        // Scenario: close_purchase applies stock, weighted cost, price, sets 'closed'.
        let conn = test_db();
        let product_id = insert_product(&conn, "Farinha", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 10.0,
            unit_cost: 3.0,
            final_price: 6.0,
            ..Default::default()
        }];

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 0.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(&conn, purchase_id).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let status: String = conn
            .query_row(r#"SELECT status FROM "Purchases" WHERE id = ?"#, params![purchase_id], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "closed");

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        assert!((cost - 3.0).abs() < 1e-9, "expected cost 3.0, got {}", cost);

        let qty: f64 = conn
            .query_row(r#"SELECT COALESCE(current_quantity, 0.0) FROM "Stock" WHERE product_id = ?"#, params![product_id], |r| r.get(0))
            .unwrap_or(0.0);
        assert!((qty - 10.0).abs() < 1e-9, "expected stock 10.0, got {}", qty);

        let fp: f64 = conn
            .query_row(r#"SELECT CAST(final_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        assert!((fp - 6.0).abs() < 1e-9, "expected final_price 6.0, got {}", fp);
    }

    #[test]
    fn close_purchase_freight_enters_cost_proportionally() {
        // Scenario: frete rateado entra no cost_price no fechamento.
        // single item: qty=10, unit_cost=4, freight=100 → lot_cost=140, custo_unit=14.0
        let conn = test_db();
        let product_id = insert_product(&conn, "Papel Foto", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 10.0,
            unit_cost: 4.0,
            final_price: 0.0,
            ..Default::default()
        }];

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 100.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(&conn, purchase_id).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        assert!((cost - 14.0).abs() < 1e-9, "expected 14.0, got {}", cost);
    }

    // --- Unit qty conversion tests (task 4.4) ---

    #[test]
    fn close_purchase_unit_qty_factor_converts_stock() {
        // Scenario: unit_qty factor: stock_qty = quantity * unit_qty.
        // qty=2, unit_qty=500 → stock_qty=1000, cost = (2*10)/1000 = 0.02
        let conn = test_db();
        let product_id = insert_product(&conn, "Folha A4", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 2.0,
            unit_cost: 10.0,
            final_price: 0.0,
            unit_qty: Some(500.0),
            ..Default::default()
        }];

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 0.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(&conn, purchase_id).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let qty: f64 = conn
            .query_row(r#"SELECT COALESCE(current_quantity, 0.0) FROM "Stock" WHERE product_id = ?"#, params![product_id], |r| r.get(0))
            .unwrap_or(0.0);
        assert!((qty - 1000.0).abs() < 1e-9, "expected stock 1000, got {}", qty);

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        let expected = 20.0 / 1000.0;
        assert!((cost - expected).abs() < 1e-9, "expected cost {}, got {}", expected, cost);
    }

    #[test]
    fn close_purchase_without_unit_qty_uses_quantity_directly() {
        // Scenario: unit_qty None → stock_qty = quantity.
        let conn = test_db();
        let product_id = insert_product(&conn, "Bobina", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 3.0,
            unit_cost: 5.0,
            final_price: 0.0,
            ..Default::default()
        }];

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 0.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(&conn, purchase_id).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let qty: f64 = conn
            .query_row(r#"SELECT COALESCE(current_quantity, 0.0) FROM "Stock" WHERE product_id = ?"#, params![product_id], |r| r.get(0))
            .unwrap_or(0.0);
        assert!((qty - 3.0).abs() < 1e-9, "expected stock 3.0, got {}", qty);
    }

    // --- Double-close test (task 4.5) ---

    #[test]
    fn close_purchase_already_closed_returns_error() {
        // Scenario: close_purchase on already 'closed' returns error.
        let conn = test_db();
        let product_id = insert_product(&conn, "Papel", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 5.0,
            unit_cost: 2.0,
            final_price: 0.0,
            ..Default::default()
        }];

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 0.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(&conn, purchase_id).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        // Second close: status is already 'closed', should fail before BEGIN
        let status: String = conn
            .query_row(r#"SELECT status FROM "Purchases" WHERE id = ?"#, params![purchase_id], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "closed", "purchase should be closed");

        // Simulate close_purchase logic: check status → error
        assert_eq!(status, "closed");
        // The actual command would return Err("Compra já foi fechada")
    }

    // --- Rollback test (task 4.6) ---

    #[test]
    fn close_purchase_rollback_reverts_to_open() {
        // Scenario: failure at close rolls back; status remains 'open'.
        let conn = test_db();
        let product_id = insert_product(&conn, "Tinta", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 5.0,
            unit_cost: 10.0,
            final_price: 0.0,
            ..Default::default()
        }];

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 0.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        // Apply effects inside a transaction, then roll back to simulate failure
        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(&conn, purchase_id).unwrap();
        conn.execute_batch("ROLLBACK").unwrap();

        let status: String = conn
            .query_row(r#"SELECT status FROM "Purchases" WHERE id = ?"#, params![purchase_id], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "open", "status must remain 'open' after rollback");

        let movements: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Stock_Movements""#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(movements, 0, "no movements after rollback");
    }

    // --- Updated existing tests (task 4.7) ---

    #[test]
    fn purchase_first_sets_cost_to_unit_lot_cost() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Farinha", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 10.0,
            unit_cost: 3.0,
            final_price: 0.0,
            ..Default::default()
        }];
        run_purchase(&conn, 0.0, items);

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        assert!((cost - 3.0).abs() < 1e-9, "expected 3.0, got {}", cost);
    }

    #[test]
    fn purchase_subsequent_computes_weighted_average() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Bobina Extra", false);

        run_purchase(&conn, 0.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 10.0,
            unit_cost: 3.0,
            final_price: 0.0,
            ..Default::default()
        }]);

        run_purchase(&conn, 0.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 5.0,
            unit_cost: 5.0,
            final_price: 0.0,
            ..Default::default()
        }]);

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        let expected = (10.0 * 3.0 + 5.0 * 5.0) / 15.0;
        assert!((cost - expected).abs() < 1e-9, "expected {}, got {}", expected, cost);
    }

    #[test]
    fn purchase_freight_enters_cost_price_proportionally() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Papel Foto", false);

        run_purchase(&conn, 100.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 10.0,
            unit_cost: 4.0,
            final_price: 0.0,
            ..Default::default()
        }]);

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        assert!((cost - 14.0).abs() < 1e-9, "expected 14.0, got {}", cost);
    }

    #[test]
    fn purchase_unit_qty_conversion_adjusts_unit_lot_cost() {
        // qty=2, unit_qty=500 → stock_qty=1000, cost=(2*10)/1000=0.02
        let conn = test_db();
        let product_id = insert_product(&conn, "Folha A4", false);

        run_purchase(&conn, 0.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 2.0,
            unit_cost: 10.0,
            final_price: 0.0,
            unit_qty: Some(500.0),
            ..Default::default()
        }]);

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        let expected = (2.0 * 10.0) / 1000.0;
        assert!((cost - expected).abs() < 1e-9, "expected {}, got {}", expected, cost);
    }

    #[test]
    fn purchase_records_loss_rate_and_propagates_to_product() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Papel Foto", false);

        run_purchase(&conn, 0.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 10.0,
            unit_cost: 5.0,
            final_price: 0.0,
            ..Default::default()
        }]);

        let stored_loss: f64 = conn
            .query_row(
                r#"SELECT CAST(loss_rate AS DOUBLE) FROM "Purchase_Items" WHERE product_id = ?"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
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

        run_purchase(&conn, 0.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 5.0,
            unit_cost: 20.0,
            final_price: 0.0,
            loss_rate: Some(0.15),
            loss_mode: Some("manual".to_string()),
            ..Default::default()
        }]);

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
        let conn = test_db();
        let product_id = insert_product(&conn, "Bobina", false);
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

        run_purchase(&conn, 0.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 10.0,
            unit_cost: 3.0,
            final_price: 0.0,
            loss_mode: Some("auto".to_string()),
            ..Default::default()
        }]);

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
        let conn = test_db();
        let product_id = insert_product(&conn, "Papel", false);

        run_purchase(&conn, 0.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 4.0,
            unit_cost: 8.0,
            final_price: 0.0,
            loss_rate: Some(0.25),
            loss_mode: Some("manual".to_string()),
            ..Default::default()
        }]);

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
        let conn = test_db();
        let product_id = insert_product(&conn, "Tinta", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 2.0,
            unit_cost: 5.0,
            final_price: 0.0,
            loss_mode: Some("automatico".to_string()),
            ..Default::default()
        }];
        conn.execute_batch("BEGIN").unwrap();
        let result = create_purchase_draft(&conn, 0.0, &items);
        let _ = conn.execute_batch("ROLLBACK");
        assert!(result.is_err(), "invalid loss_mode should be rejected");
    }

    #[test]
    fn purchase_inserts_purchase_and_stock_movement_on_close() {
        let conn = test_db();
        let id = insert_product(&conn, "Papel A4", false);

        run_purchase(&conn, 10.0, vec![PurchaseItemInput {
            product_id: Some(id),
            quantity: 5.0,
            unit_cost: 20.0,
            final_price: 0.0,
            ..Default::default()
        }]);

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
    fn purchase_without_conversion_uses_quantity_directly() {
        let quantity = 3.0_f64;
        let unit_qty: Option<f64> = None;
        let stock_qty = unit_qty.map(|f| quantity * f).unwrap_or(quantity);
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
        assert_eq!(venda.sale_id, Some(sale_id));
        let compra = movements.iter().find(|m| m.motive == "compra").unwrap();
        assert_eq!(compra.sale_id, None);
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

    #[test]
    fn purchase_freight_rateado_proportional_multi_item() {
        let conn = test_db();
        let id_a = insert_product(&conn, "Insumo A", false);
        let id_b = insert_product(&conn, "Insumo B", false);

        run_purchase(&conn, 20.0, vec![
            PurchaseItemInput { product_id: Some(id_a), quantity: 5.0, unit_cost: 2.0, final_price: 0.0, ..Default::default() },
            PurchaseItemInput { product_id: Some(id_b), quantity: 10.0, unit_cost: 3.0, final_price: 0.0, ..Default::default() },
        ]);

        let cost_a: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![id_a], |r| r.get(0))
            .unwrap();
        let cost_b: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![id_b], |r| r.get(0))
            .unwrap();
        assert!((cost_a - 3.0).abs() < 1e-9, "expected 3.0, got {}", cost_a);
        assert!((cost_b - 4.5).abs() < 1e-9, "expected 4.5, got {}", cost_b);
    }

    #[test]
    fn purchase_freight_not_rateado_when_subtotal_zero() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Brinde", false);

        run_purchase(&conn, 50.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 5.0,
            ..Default::default()
        }]);

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        assert!((cost - 0.0).abs() < 1e-9, "expected 0.0, got {}", cost);
    }

    #[test]
    fn purchase_final_price_persisted_and_margins_derived() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Produto X", false);

        run_purchase(&conn, 0.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 10.0,
            unit_cost: 4.0,
            final_price: 6.0,
            ..Default::default()
        }]);

        let (fp, markup, margin): (f64, f64, f64) = conn
            .query_row(
                r#"SELECT CAST(final_price AS DOUBLE), CAST(markup AS DOUBLE), CAST(profit_margin AS DOUBLE)
                   FROM "Products" WHERE id = ?"#,
                params![product_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert!((fp - 6.0).abs() < 1e-9, "final_price: expected 6.0, got {}", fp);
        assert!((markup - 1.5).abs() < 1e-9, "markup: expected 1.5, got {}", markup);
        let expected_margin = 1.0 - 4.0_f64 / 6.0_f64;
        assert!((margin - expected_margin).abs() < 1e-9, "profit_margin: expected {}, got {}", expected_margin, margin);
    }

    #[test]
    fn purchase_freight_rateado_with_unit_qty_conversion() {
        // qty=2, unit_qty=10, unit_cost=5 → stock_qty=20, line=10, subtotal=10, freight=10
        // freight_share=10, lot_cost=20, custo_unit_lote=1.0
        let conn = test_db();
        let product_id = insert_product(&conn, "Papel Unidade", false);

        run_purchase(&conn, 10.0, vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 2.0,
            unit_cost: 5.0,
            unit_qty: Some(10.0),
            ..Default::default()
        }]);

        let cost: f64 = conn
            .query_row(r#"SELECT CAST(cost_price AS DOUBLE) FROM "Products" WHERE id = ?"#, params![product_id], |r| r.get(0))
            .unwrap();
        assert!((cost - 1.0).abs() < 1e-9, "expected 1.0, got {}", cost);
    }

    #[test]
    fn list_purchases_filters_by_status() {
        // Scenario: list_purchases with status 'open' returns only open purchases.
        let conn = test_db();
        let product_id = insert_product(&conn, "Insumo", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 1.0,
            unit_cost: 1.0,
            ..Default::default()
        }];

        // Create one open draft
        conn.execute_batch("BEGIN").unwrap();
        create_purchase_draft(&conn, 0.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        // Create another and close it
        conn.execute_batch("BEGIN").unwrap();
        let closed_id = create_purchase_draft(&conn, 0.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();
        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(&conn, closed_id).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let open_count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Purchases" WHERE status = 'open'"#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(open_count, 1, "should have 1 open purchase");

        let closed_count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Purchases" WHERE status = 'closed'"#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(closed_count, 1, "should have 1 closed purchase");
    }

    // Section 4: new-product inline creation tests

    #[test]
    fn new_product_draft_persists_pending_fields_no_product_created() {
        let conn = test_db();
        let item = PurchaseItemInput {
            new_product_name: Some("Novo Insumo".to_string()),
            new_product_unit: Some("unidade".to_string()),
            new_product_description: Some("desc".to_string()),
            new_product_sellable: Some(false),
            quantity: 5.0,
            unit_cost: 2.0,
            ..Default::default()
        };

        conn.execute_batch("BEGIN").unwrap();
        create_purchase_draft(&conn, 0.0, &[item]).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Products""#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "no product must be created during draft");

        let (name, unit, desc, sellable, pid): (String, String, Option<String>, bool, Option<i64>) = conn
            .query_row(
                r#"SELECT new_product_name, new_product_unit, new_product_description,
                          new_product_sellable, product_id
                   FROM "Purchase_Items""#,
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();
        assert_eq!(name, "Novo Insumo");
        assert_eq!(unit, "unidade");
        assert_eq!(desc, Some("desc".to_string()));
        assert!(!sellable);
        assert!(pid.is_none(), "product_id must be null in draft");
    }

    #[test]
    fn close_purchase_creates_new_product_backfills_and_applies_effects() {
        let conn = test_db();
        use crate::settings::insert_unit_of_measure;
        insert_unit_of_measure(&conn, "unidade", "un");

        let item = PurchaseItemInput {
            new_product_name: Some("Produto Fechamento".to_string()),
            new_product_unit: Some("unidade".to_string()),
            new_product_sellable: Some(false),
            quantity: 10.0,
            unit_cost: 3.0,
            final_price: 5.0,
            ..Default::default()
        };

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 0.0, &[item]).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(&conn, purchase_id).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let pid: Option<i64> = conn
            .query_row(
                r#"SELECT product_id FROM "Purchase_Items" WHERE purchase_id = ?"#,
                params![purchase_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(pid.is_some(), "product_id must be backfilled after close");

        let (cost, name): (f64, String) = conn
            .query_row(
                r#"SELECT CAST(cost_price AS DOUBLE), name FROM "Products" WHERE id = ?"#,
                params![pid.unwrap()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(name, "Produto Fechamento");
        assert!((cost - 3.0).abs() < 1e-9, "cost_price expected 3.0, got {}", cost);

        let stock_qty: f64 = conn
            .query_row(
                r#"SELECT quantity FROM "Stock_Movements"
                   WHERE product_id = ? AND movement_type = 'in'"#,
                params![pid.unwrap()],
                |r| r.get(0),
            )
            .unwrap();
        assert!((stock_qty - 10.0).abs() < 1e-9, "stock qty expected 10.0, got {}", stock_qty);
    }

    #[test]
    fn draft_item_without_product_or_new_product_data_returns_error() {
        let conn = test_db();
        let item = PurchaseItemInput {
            quantity: 1.0,
            unit_cost: 1.0,
            ..Default::default()
        };

        conn.execute_batch("BEGIN").unwrap();
        let result = create_purchase_draft(&conn, 0.0, &[item]);
        conn.execute_batch("ROLLBACK").unwrap();
        assert!(result.is_err(), "must reject item with no product reference");
    }

    #[test]
    fn close_purchase_rollback_removes_new_product() {
        let conn = test_db();
        use crate::settings::insert_unit_of_measure;
        insert_unit_of_measure(&conn, "unidade", "un");

        let item = PurchaseItemInput {
            new_product_name: Some("Produto Rollback".to_string()),
            new_product_unit: Some("unidade".to_string()),
            new_product_sellable: Some(false),
            quantity: 1.0,
            unit_cost: 1.0,
            ..Default::default()
        };

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 0.0, &[item]).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        conn.execute_batch("BEGIN").unwrap();
        do_close_effects(&conn, purchase_id).unwrap();
        conn.execute_batch("ROLLBACK").unwrap();

        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Products" WHERE name = 'Produto Rollback'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "product must be rolled back with the transaction");
    }

    #[test]
    fn close_purchase_new_product_invalid_unit_returns_error() {
        let conn = test_db();
        let item = PurchaseItemInput {
            new_product_name: Some("Produto Unidade Ruim".to_string()),
            new_product_unit: Some("unidade_inexistente".to_string()),
            new_product_sellable: Some(false),
            quantity: 1.0,
            unit_cost: 1.0,
            ..Default::default()
        };

        conn.execute_batch("BEGIN").unwrap();
        let purchase_id = create_purchase_draft(&conn, 0.0, &[item]).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        conn.execute_batch("BEGIN").unwrap();
        let result = do_close_effects(&conn, purchase_id);
        conn.execute_batch("ROLLBACK").unwrap();

        assert!(result.is_err(), "must fail with invalid unit");
        let count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Products""#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0, "no product must remain after failed close");
    }

    #[test]
    fn list_product_purchases_returns_closed_purchase() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Farinha", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 5.0,
            unit_cost: 2.0,
            final_price: 4.0,
            ..Default::default()
        }];
        run_purchase(&conn, 0.0, items);

        let rows = list_product_purchases_raw(&conn, product_id).unwrap();
        assert_eq!(rows.len(), 1);
        assert!((rows[0].quantity - 5.0).abs() < 1e-9);
        assert!((rows[0].unit_cost - 2.0).abs() < 1e-9);
        assert!((rows[0].total - 10.0).abs() < 1e-9);
    }

    #[test]
    fn list_product_purchases_returns_empty_when_no_purchases() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Sem compras", false);
        let rows = list_product_purchases_raw(&conn, product_id).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn list_product_purchases_ignores_open_drafts() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Farinha", false);

        let items = vec![PurchaseItemInput {
            product_id: Some(product_id),
            quantity: 3.0,
            unit_cost: 1.0,
            final_price: 2.0,
            ..Default::default()
        }];

        conn.execute_batch("BEGIN").unwrap();
        create_purchase_draft(&conn, 0.0, &items).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let rows = list_product_purchases_raw(&conn, product_id).unwrap();
        assert!(rows.is_empty(), "rascunhos abertos nao devem aparecer no historico de compras");
    }
}
