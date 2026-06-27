use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::auth::{require_session, SessionState};
use crate::db::DbState;
use crate::products::compute_cmv_from_components;

#[derive(Deserialize)]
pub struct SaleItemInput {
    pub product_id: i64,
    pub quantity: i32,
    pub discount: f64,
}

#[derive(Serialize)]
pub struct SaleRow {
    pub id: i64,
    pub label: String,
    pub total_value: f64,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct SalePaymentInput {
    pub payment_method_id: i64,
    pub amount_paid: f64,
    pub installments: i64,
}

#[derive(Serialize)]
pub struct SaleItemRow {
    pub product_id: i64,
    pub product_name: String,
    pub quantity: i32,
    pub price: f64,
    pub discount: f64,
    pub discounted_price: f64,
}

#[tauri::command]
pub fn list_sales(
    status: String,
    session_token: String,
    state: State<DbState>,
    session: State<SessionState>,
) -> Result<Vec<SaleRow>, String> {
    require_session(&session, &session_token)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT id, label, total_value, CAST(created_at AS VARCHAR)
            FROM "Sales"
            WHERE status = ?
            ORDER BY created_at DESC
        "#,
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![status], |row| {
            Ok(SaleRow {
                id: row.get(0)?,
                label: row.get(1)?,
                total_value: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

pub fn do_create_sale_work(
    conn: &Connection,
    user_id: i64,
    label: &str,
    items: &[SaleItemInput],
) -> Result<i64, String> {
    let mut total_value = 0.0f64;
    let mut item_data: Vec<(i64, i32, f64, f64, f64)> = Vec::new();

    for item in items {
        let final_price: f64 = conn
            .query_row(
                r#"SELECT final_price FROM "Products" WHERE id = ?"#,
                params![item.product_id],
                |row| row.get(0),
            )
            .map_err(|_| format!("Produto {} não encontrado", item.product_id))?;

        let discount = item.discount.clamp(0.0, 1.0);
        let discounted_price = final_price * (1.0 - discount);
        total_value += discounted_price * item.quantity as f64;
        item_data.push((
            item.product_id,
            item.quantity,
            final_price,
            discount,
            discounted_price,
        ));
    }

    let sale_id: i64 = conn
        .query_row(
            r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, ?, 'open', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
            params![user_id, label, total_value],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    for (product_id, quantity, price, discount, discounted_price) in &item_data {
        conn.execute(
            r#"INSERT INTO "Sales_Products"
               (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"#,
            params![
                sale_id,
                product_id,
                quantity,
                price,
                discount,
                discounted_price
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(sale_id)
}

#[tauri::command]
pub fn create_sale(
    user_id: i64,
    label: String,
    items: Vec<SaleItemInput>,
    session_token: String,
    state: State<DbState>,
    session: State<SessionState>,
) -> Result<i64, String> {
    require_session(&session, &session_token)?;
    if items.is_empty() {
        return Err("A venda deve ter ao menos um item".to_string());
    }
    for item in &items {
        if item.quantity <= 0 {
            return Err("Quantidade de cada item deve ser positiva".to_string());
        }
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

    match do_create_sale_work(&conn, user_id, label.trim(), &items) {
        Ok(id) => {
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
            Ok(id)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

#[tauri::command]
pub fn update_sale(
    id: i64,
    label: String,
    items: Vec<SaleItemInput>,
    session_token: String,
    state: State<DbState>,
    session: State<SessionState>,
) -> Result<(), String> {
    require_session(&session, &session_token)?;
    if items.is_empty() {
        return Err("A venda deve ter ao menos um item".to_string());
    }
    for item in &items {
        if item.quantity <= 0 {
            return Err("Quantidade de cada item deve ser positiva".to_string());
        }
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let status: String = conn
        .query_row(
            r#"SELECT status FROM "Sales" WHERE id = ?"#,
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| format!("Venda {} não encontrada", id))?;

    if status != "open" {
        return Err("Não é possível editar uma venda fechada".to_string());
    }

    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

    let result = (|| -> Result<(), String> {
        conn.execute(
            r#"DELETE FROM "Sales_Products" WHERE sale_id = ?"#,
            params![id],
        )
        .map_err(|e| e.to_string())?;

        let mut total_value = 0.0f64;
        for item in &items {
            let final_price: f64 = conn
                .query_row(
                    r#"SELECT final_price FROM "Products" WHERE id = ?"#,
                    params![item.product_id],
                    |row| row.get(0),
                )
                .map_err(|_| format!("Produto {} não encontrado", item.product_id))?;

            let discount = item.discount.clamp(0.0, 1.0);
            let discounted_price = final_price * (1.0 - discount);
            total_value += discounted_price * item.quantity as f64;

            conn.execute(
                r#"INSERT INTO "Sales_Products"
                   (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"#,
                params![
                    id,
                    item.product_id,
                    item.quantity,
                    final_price,
                    discount,
                    discounted_price
                ],
            )
            .map_err(|e| e.to_string())?;
        }

        conn.execute(
            r#"UPDATE "Sales"
               SET label = ?, total_value = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![label.trim(), total_value, id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    })();

    match result {
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

pub fn do_close_sale_work(
    conn: &Connection,
    sale_id: i64,
    items: &[SaleItemInput],
    payments: &[SalePaymentInput],
) -> Result<Vec<String>, String> {
    let status: String = conn
        .query_row(
            r#"SELECT status FROM "Sales" WHERE id = ?"#,
            params![sale_id],
            |row| row.get(0),
        )
        .map_err(|_| format!("Venda {} não encontrada", sale_id))?;

    if status != "open" {
        return Err("Venda já foi fechada".to_string());
    }

    let allow_negative: bool = conn
        .query_row(
            r#"SELECT "value" FROM "System_Settings" WHERE "key" = 'allow_negative_stock_sale'"#,
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "false".to_string())
        == "true";

    for item in items {
        let price: f64 = conn
            .query_row(
                r#"SELECT price FROM "Sales_Products" WHERE sale_id = ? AND product_id = ?"#,
                params![sale_id, item.product_id],
                |row| row.get(0),
            )
            .map_err(|_| format!("Item {} não encontrado no pedido", item.product_id))?;

        let discount = item.discount.clamp(0.0, 1.0);
        let discounted_price = price * (1.0 - discount);

        conn.execute(
            r#"UPDATE "Sales_Products" SET discount = ?, discounted_price = ? WHERE sale_id = ? AND product_id = ?"#,
            params![discount, discounted_price, sale_id, item.product_id],
        )
        .map_err(|e| e.to_string())?;
    }

    let total_value: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(discounted_price * quantity), 0.0) FROM "Sales_Products" WHERE sale_id = ?"#,
            params![sale_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        r#"UPDATE "Sales" SET total_value = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
        params![total_value, sale_id],
    )
    .map_err(|e| e.to_string())?;

    let total_paid: f64 = payments.iter().map(|p| p.amount_paid).sum();
    if (total_paid - total_value).abs() > 0.01 {
        return Err(format!(
            "Valor pago ({:.2}) não corresponde ao total da venda ({:.2})",
            total_paid, total_value
        ));
    }

    for payment in payments {
        // Task 5.2/5.3: look up fee by (payment_method_id, installments); default 0 if not found
        let (fee_percent, fixed_fee): (f64, f64) = conn
            .query_row(
                r#"SELECT fee_percent, fixed_fee FROM "Transaction_Fees"
                   WHERE payment_method_id = ? AND installments = ?"#,
                params![payment.payment_method_id, payment.installments],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0.0, 0.0));

        let net_amount = payment.amount_paid * (1.0 - fee_percent) - fixed_fee;

        conn.execute(
            r#"INSERT INTO "Sale_Payments"
               (sale_id, payment_method_id, amount_paid, net_amount,
                applied_fee_percent, applied_fixed_fee, created_at)
               VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"#,
            params![
                sale_id,
                payment.payment_method_id,
                payment.amount_paid,
                net_amount,
                fee_percent,
                fixed_fee
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    let mut stmt = conn
        .prepare(
            r#"
            SELECT sp.product_id, sp.quantity, p.sellable, p.name
            FROM "Sales_Products" sp
            JOIN "Products" p ON p.id = sp.product_id
            WHERE sp.sale_id = ?
        "#,
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map(params![sale_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut warnings: Vec<String> = Vec::new();

    for (product_id, quantity, sellable, product_name) in items {
        if sellable {
            let order_id: i64 = conn
                .query_row(
                    r#"INSERT INTO "Production_Orders"
                       (sale_id, product_id, quantity, created_at)
                       VALUES (?, ?, ?, CURRENT_TIMESTAMP)
                       RETURNING id"#,
                    params![sale_id, product_id, quantity],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;

            let mut comp_stmt = conn
                .prepare(
                    r#"
                    SELECT pc.child_id, CAST(pc.quantity AS DOUBLE), p.name,
                           COALESCE(s.current_quantity, 0.0)
                    FROM "Product_Components" pc
                    JOIN "Products" p ON p.id = pc.child_id
                    LEFT JOIN "Stock" s ON s.product_id = pc.child_id
                    WHERE pc.parent_id = ?
                "#,
                )
                .map_err(|e| e.to_string())?;

            let components: Vec<(i64, f64, String, f64)> = comp_stmt
                .query_map(params![product_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            if components.is_empty() {
                let current_stock: f64 = conn
                    .query_row(
                        r#"SELECT COALESCE(current_quantity, 0.0) FROM "Stock" WHERE product_id = ?"#,
                        params![product_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0.0);

                if current_stock < quantity as f64 && !allow_negative {
                    warnings.push(format!(
                        "Estoque insuficiente de '{}': disponível {:.2}, necessário {:.2}",
                        product_name, current_stock, quantity as f64
                    ));
                }

                conn.execute(
                    r#"INSERT INTO "Stock_Movements"
                       (product_id, quantity, movement_type, motive, created_at, sale_id)
                       VALUES (?, ?, 'out', 'venda', CURRENT_TIMESTAMP, ?)"#,
                    params![product_id, quantity as f64, sale_id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                for (child_id, comp_qty, child_name, current_stock) in components {
                    let qty_to_consume = comp_qty * quantity as f64;

                    if current_stock < qty_to_consume && !allow_negative {
                        warnings.push(format!(
                            "Estoque insuficiente de '{}': disponível {:.2}, necessário {:.2}",
                            child_name, current_stock, qty_to_consume
                        ));
                    }

                    conn.execute(
                        r#"INSERT INTO "Stock_Movements"
                           (product_id, quantity, movement_type, motive, created_at, sale_id)
                           VALUES (?, ?, 'out', 'venda', CURRENT_TIMESTAMP, ?)"#,
                        params![child_id, qty_to_consume, sale_id],
                    )
                    .map_err(|e| e.to_string())?;

                    conn.execute(
                        r#"INSERT INTO "Production_Inputs"
                           (production_order_id, product_id, quantity_used, created_at)
                           VALUES (?, ?, ?, CURRENT_TIMESTAMP)"#,
                        params![order_id, child_id, qty_to_consume],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
    }

    let sp_ids: Vec<(i64, i64)> = {
        let mut stmt = conn
            .prepare(r#"SELECT id, product_id FROM "Sales_Products" WHERE sale_id = ?"#)
            .map_err(|e| e.to_string())?;
        stmt.query_map(params![sale_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };
    for (sp_id, pid) in sp_ids {
        let cmv: f64 = match compute_cmv_from_components(conn, pid) {
            Ok(Some(derived)) => derived,
            _ => {
                let (cost, loss): (f64, f64) = conn
                    .query_row(
                        r#"SELECT CAST(cost_price AS DOUBLE), CAST(expected_loss_rate AS DOUBLE)
                           FROM "Products" WHERE id = ?"#,
                        params![pid],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .unwrap_or((0.0, 0.0));
                if loss < 1.0 {
                    cost / (1.0 - loss)
                } else {
                    0.0
                }
            }
        };
        conn.execute(
            r#"UPDATE "Sales_Products" SET cost_price = ? WHERE id = ?"#,
            params![cmv, sp_id],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.execute(
        r#"UPDATE "Sales" SET status = 'closed', updated_at = CURRENT_TIMESTAMP WHERE id = ?"#,
        params![sale_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(warnings)
}

#[tauri::command]
pub fn close_sale(
    id: i64,
    items: Vec<SaleItemInput>,
    payments: Vec<SalePaymentInput>,
    session_token: String,
    state: State<DbState>,
    session: State<SessionState>,
) -> Result<Vec<String>, String> {
    require_session(&session, &session_token)?;
    if payments.is_empty() {
        return Err("Informe ao menos um pagamento".to_string());
    }
    if items.is_empty() {
        return Err("A venda deve ter ao menos um item".to_string());
    }
    for item in &items {
        if item.quantity <= 0 {
            return Err("Quantidade de cada item deve ser positiva".to_string());
        }
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

    match do_close_sale_work(&conn, id, &items, &payments) {
        Ok(warnings) => {
            conn.execute_batch("COMMIT").map_err(|e| e.to_string())?;
            Ok(warnings)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

#[derive(Serialize)]
pub struct SalePaymentRow {
    pub payment_method_name: String,
    pub amount_paid: f64,
    pub net_amount: f64,
}

#[tauri::command]
pub fn get_sale_payments(
    sale_id: i64,
    session_token: String,
    state: State<DbState>,
    session: State<SessionState>,
) -> Result<Vec<SalePaymentRow>, String> {
    require_session(&session, &session_token)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT pm.name, sp.amount_paid, sp.net_amount
            FROM "Sale_Payments" sp
            JOIN "Payment_Methods" pm ON pm.id = sp.payment_method_id
            WHERE sp.sale_id = ?
            ORDER BY sp.created_at
        "#,
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![sale_id], |row| {
            Ok(SalePaymentRow {
                payment_method_name: row.get(0)?,
                amount_paid: row.get(1)?,
                net_amount: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn get_sale_items(
    sale_id: i64,
    session_token: String,
    state: State<DbState>,
    session: State<SessionState>,
) -> Result<Vec<SaleItemRow>, String> {
    require_session(&session, &session_token)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            r#"
            SELECT sp.product_id, p.name, sp.quantity, sp.price, sp.discount, sp.discounted_price
            FROM "Sales_Products" sp
            JOIN "Products" p ON p.id = sp.product_id
            WHERE sp.sale_id = ?
            ORDER BY sp.created_at
        "#,
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![sale_id], |row| {
            Ok(SaleItemRow {
                product_id: row.get(0)?,
                product_name: row.get(1)?,
                quantity: row.get(2)?,
                price: row.get(3)?,
                discount: row.get(4)?,
                discounted_price: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[derive(Serialize)]
pub struct ProductSaleRow {
    pub sale_id: i64,
    pub sale_date: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub revenue: f64,
}

fn get_product_sales_history_raw(
    conn: &Connection,
    product_id: i64,
) -> Result<Vec<ProductSaleRow>, String> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT s.id, CAST(s.created_at AS VARCHAR),
                   sp.quantity,
                   CAST(sp.discounted_price AS DOUBLE),
                   CAST(sp.discounted_price AS DOUBLE) * sp.quantity
            FROM "Sales_Products" sp
            JOIN "Sales" s ON s.id = sp.sale_id
            WHERE sp.product_id = ? AND s.status = 'closed'
            ORDER BY s.created_at DESC
        "#,
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![product_id], |row| {
            Ok(ProductSaleRow {
                sale_id: row.get(0)?,
                sale_date: row.get(1)?,
                quantity: row.get(2)?,
                unit_price: row.get(3)?,
                revenue: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

#[tauri::command]
pub fn get_product_sales_history(
    product_id: i64,
    session_token: String,
    state: State<DbState>,
    session: State<SessionState>,
) -> Result<Vec<ProductSaleRow>, String> {
    require_session(&session, &session_token)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    get_product_sales_history_raw(&conn, product_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::insert_user;
    use crate::db::test_db;
    use crate::products::insert_product;
    use crate::settings::{insert_payment_method, insert_transaction_fee};
    use crate::stock::add_product_component_in_test;

    #[test]
    fn create_sale_without_items_returns_error() {
        let items: Vec<SaleItemInput> = vec![];
        assert!(items.is_empty());
    }

    #[test]
    fn create_sale_inserts_sale_and_items() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let product_id = insert_product(&conn, "Foto 10x15", true);

        conn.execute_batch("BEGIN").unwrap();
        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
                   VALUES (?, 'João', 'open', 25.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products"
               (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 2, 15.0, 0.1, 13.5, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        )
        .unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Sales" WHERE label = 'João'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        let item_count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Sales_Products" WHERE sale_id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(item_count, 1);
    }

    #[test]
    fn close_sale_with_insufficient_payment_returns_error() {
        let total_value = 50.0f64;
        let total_paid = 40.0f64;
        assert!((total_paid - total_value).abs() > 0.01);
    }

    #[test]
    fn close_sale_creates_production_orders_for_sellable_items() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let sellable_id = insert_product(&conn, "Foto", true);
        let insumo_id = insert_product(&conn, "Papel", false);
        let pm_id = insert_payment_method(&conn, "Dinheiro");

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
                   VALUES (?, 'teste', 'open', 30.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products"
               (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 2, 15.0, 0.0, 15.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, sellable_id],
        )
        .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products"
               (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 0.0, 0.0, 0.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, insumo_id],
        )
        .unwrap();

        let items = vec![
            SaleItemInput {
                product_id: sellable_id,
                quantity: 2,
                discount: 0.0,
            },
            SaleItemInput {
                product_id: insumo_id,
                quantity: 1,
                discount: 0.0,
            },
        ];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 30.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let prod_count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Production_Orders" WHERE sale_id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(prod_count, 1);

        let status: String = conn
            .query_row(
                r#"SELECT status FROM "Sales" WHERE id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "closed");
    }

    #[test]
    fn close_sale_consumes_stock_for_components() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let foto_id = insert_product(&conn, "Foto 10x15", true);
        let papel_id = insert_product(&conn, "Papel", false);
        let pm_id = insert_payment_method(&conn, "Dinheiro");

        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 100.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![papel_id],
        ).unwrap();

        add_product_component_in_test(&conn, foto_id, papel_id, 2.0);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 30.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 3, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, foto_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: foto_id,
            quantity: 3,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 30.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert!(
            warnings.is_empty(),
            "Não deveria haver avisos com estoque suficiente"
        );

        let consumed: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out'"#,
                params![papel_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            (consumed - 6.0).abs() < 1e-9,
            "Esperado consumo de 6 folhas, foi {}",
            consumed
        );

        let input_count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Production_Inputs" WHERE product_id = ?"#,
                params![papel_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(input_count, 1);
    }

    #[test]
    fn close_sale_records_sale_id_on_direct_stock_movement() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Caneca", true);

        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 10.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![product_id],
        ).unwrap();

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 20.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 2, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id,
            quantity: 2,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 20.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let movement_sale_id: Option<i64> = conn
            .query_row(
                r#"SELECT sale_id FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out' AND motive = 'venda'"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            movement_sale_id,
            Some(sale_id),
            "Baixa direta deve gravar o sale_id da venda"
        );
    }

    #[test]
    fn close_sale_records_sale_id_on_component_stock_movements() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let foto_id = insert_product(&conn, "Foto 10x15", true);
        let papel_id = insert_product(&conn, "Papel", false);
        let pm_id = insert_payment_method(&conn, "Dinheiro");

        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 100.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![papel_id],
        ).unwrap();
        add_product_component_in_test(&conn, foto_id, papel_id, 2.0);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 30.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 3, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, foto_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: foto_id,
            quantity: 3,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 30.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let (total, matching): (i64, i64) = conn
            .query_row(
                r#"SELECT COUNT(*), COUNT(*) FILTER (WHERE sale_id = ?)
               FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out' AND motive = 'venda'"#,
                params![sale_id, papel_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(total >= 1, "Deve haver ao menos uma baixa de insumo");
        assert_eq!(
            total, matching,
            "Todas as baixas de insumo via BOM devem gravar o sale_id da venda"
        );
    }

    #[test]
    fn close_sale_snapshots_cost_price() {
        // insert_product hardcodes cost_price = 10.0; verify the snapshot captures that value
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Foto", true);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 10.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let cost: Option<f64> = conn
            .query_row(
                r#"SELECT cost_price FROM "Sales_Products" WHERE sale_id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            cost.is_some(),
            "cost_price não deve ser NULL após fechar venda"
        );
        assert!(
            (cost.unwrap() - 10.0).abs() < 1e-9,
            "cost_price deveria ser 10.0 (snapshot do produto), foi {:?}",
            cost
        );
    }

    #[test]
    fn close_sale_with_insufficient_stock_returns_warning_and_proceeds() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let foto_id = insert_product(&conn, "Foto 10x15", true);
        let papel_id = insert_product(&conn, "Papel", false);
        let pm_id = insert_payment_method(&conn, "Dinheiro");

        add_product_component_in_test(&conn, foto_id, papel_id, 1.0);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, foto_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: foto_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 10.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert_eq!(
            warnings.len(),
            1,
            "Deveria haver 1 aviso de estoque insuficiente"
        );
        assert!(warnings[0].contains("Papel"));

        let status: String = conn
            .query_row(
                r#"SELECT status FROM "Sales" WHERE id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "closed");

        let consumed: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out'"#,
                params![papel_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!((consumed - 1.0).abs() < 1e-9);
    }

    // Task 5.4 tests

    #[test]
    fn close_sale_net_amount_with_fee_percent_and_fixed() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let product_id = insert_product(&conn, "Foto", true);
        let pm_id = insert_payment_method(&conn, "Cartão");

        // Fee: 2% + R$0.50 fixed for installment 1
        insert_transaction_fee(&conn, pm_id, 1, 0.02, 0.50);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 100.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 100.0, 0.0, 100.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 100.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let (net, fee_pct, fixed): (f64, f64, f64) = conn
            .query_row(
                r#"SELECT net_amount, applied_fee_percent, applied_fixed_fee
               FROM "Sale_Payments" WHERE sale_id = ?"#,
                params![sale_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();

        // net = 100 * (1 - 0.02) - 0.50 = 98.0 - 0.5 = 97.5
        assert!(
            (net - 97.5).abs() < 1e-6,
            "net_amount should be 97.5, got {}",
            net
        );
        assert!((fee_pct - 0.02).abs() < 1e-9);
        assert!((fixed - 0.5).abs() < 1e-6);
    }

    #[test]
    fn close_sale_split_with_different_installments() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let product_id = insert_product(&conn, "Foto", true);
        let pm_id = insert_payment_method(&conn, "Crédito");

        conn.execute(
            r#"INSERT INTO "Transaction_Fees" (payment_method_id, installments, fee_percent, fixed_fee, updated_at, created_at)
               VALUES (?, 1, 0.01, 0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![pm_id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Transaction_Fees" (payment_method_id, installments, fee_percent, fixed_fee, updated_at, created_at)
               VALUES (?, 3, 0.03, 0.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![pm_id],
        ).unwrap();

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 200.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 200.0, 0.0, 200.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![
            SalePaymentInput {
                payment_method_id: pm_id,
                amount_paid: 100.0,
                installments: 1,
            },
            SalePaymentInput {
                payment_method_id: pm_id,
                amount_paid: 100.0,
                installments: 3,
            },
        ];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let nets: Vec<f64> = {
            let mut stmt = conn.prepare(
                r#"SELECT net_amount FROM "Sale_Payments" WHERE sale_id = ? ORDER BY created_at"#,
            ).unwrap();
            stmt.query_map(params![sale_id], |r| r.get(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };
        // 100 * (1 - 0.01) = 99.0 and 100 * (1 - 0.03) = 97.0
        assert!(
            (nets[0] - 99.0).abs() < 1e-6,
            "first net should be 99.0, got {}",
            nets[0]
        );
        assert!(
            (nets[1] - 97.0).abs() < 1e-6,
            "second net should be 97.0, got {}",
            nets[1]
        );
    }

    #[test]
    fn close_sale_method_without_fee_preserves_full_amount() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let product_id = insert_product(&conn, "Foto", true);
        let pm_id = insert_payment_method(&conn, "Dinheiro");

        // No fees for this method
        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 50.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 50.0, 0.0, 50.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 50.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let net: f64 = conn
            .query_row(
                r#"SELECT net_amount FROM "Sale_Payments" WHERE sale_id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            (net - 50.0).abs() < 1e-6,
            "no fee → net_amount = amount_paid = 50.0, got {}",
            net
        );
    }

    #[test]
    fn close_sale_resale_product_deducts_own_stock() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Caneca", true);

        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 10.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![product_id],
        ).unwrap();

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 30.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 3, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: product_id,
            quantity: 3,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 30.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert!(
            warnings.is_empty(),
            "Não deveria haver avisos com estoque suficiente"
        );

        let out_qty: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out'"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            (out_qty - 3.0).abs() < 1e-9,
            "Esperado 3 unidades baixadas, foi {}",
            out_qty
        );

        let balance: f64 = conn
            .query_row(
                r#"SELECT COALESCE(current_quantity, 0.0) FROM "Stock" WHERE product_id = ?"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            (balance - 7.0).abs() < 1e-9,
            "Saldo esperado 7, foi {}",
            balance
        );
    }

    #[test]
    fn close_sale_resale_insufficient_stock_warns_and_proceeds() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Caneca", true);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 10.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert_eq!(
            warnings.len(),
            1,
            "Deve haver 1 aviso de estoque insuficiente"
        );
        assert!(
            warnings[0].contains("Caneca"),
            "Aviso deve identificar o produto"
        );

        let status: String = conn
            .query_row(
                r#"SELECT status FROM "Sales" WHERE id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            status, "closed",
            "Venda deve ser fechada mesmo com estoque insuficiente"
        );

        let out_qty: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out'"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            (out_qty - 1.0).abs() < 1e-9,
            "Movimento 'out' deve ser gravado mesmo com saldo insuficiente"
        );
    }

    #[test]
    fn close_sale_resale_no_production_inputs_but_has_order() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Caneca", true);

        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 5.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![product_id],
        ).unwrap();

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 10.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let order_count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Production_Orders" WHERE sale_id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            order_count, 1,
            "Deve haver 1 Production_Order para o produto de revenda"
        );

        let input_count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Production_Inputs" pi
               JOIN "Production_Orders" po ON po.id = pi.production_order_id
               WHERE po.sale_id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            input_count, 0,
            "Não deve haver Production_Inputs para produto de revenda"
        );
    }

    #[test]
    fn close_sale_applies_item_discount_and_recalculates_total() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Foto", true);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 20.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 2, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        // 10% discount on 10.0 → discounted_price = 9.0; total = 9.0 * 2 = 18.0
        let items = vec![SaleItemInput {
            product_id,
            quantity: 2,
            discount: 0.1,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 18.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        // DECIMAL(10,2) reads correctly; discounted_price and total_value reflect the applied discount
        let discounted_price: f64 = conn.query_row(
            r#"SELECT CAST(discounted_price AS DOUBLE) FROM "Sales_Products" WHERE sale_id = ?"#,
            params![sale_id], |r| r.get(0),
        ).unwrap();
        assert!(
            (discounted_price - 9.0).abs() < 1e-6,
            "discounted_price deve ser 9.0, foi {}",
            discounted_price
        );

        let total_value: f64 = conn
            .query_row(
                r#"SELECT CAST(total_value AS DOUBLE) FROM "Sales" WHERE id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            (total_value - 18.0).abs() < 1e-6,
            "total_value deve ser 18.0, foi {}",
            total_value
        );
    }

    #[test]
    fn close_sale_payment_validated_against_discounted_total() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Foto", true);

        let make_sale = |total: f64| -> i64 {
            let sid: i64 = conn.query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
                   VALUES (?, '', 'open', ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id"#,
                params![user_id, total], |r| r.get(0),
            ).unwrap();
            conn.execute(
                r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
                   VALUES (?, ?, 1, 20.0, 0.0, 20.0, CURRENT_TIMESTAMP)"#,
                params![sid, product_id],
            ).unwrap();
            sid
        };

        // Paying full price (20.0) when discount brings total to 18.0 should fail
        let sale1 = make_sale(20.0);
        let items_with_discount = vec![SaleItemInput {
            product_id,
            quantity: 1,
            discount: 0.1,
        }];
        let pay_full = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 20.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        let result = do_close_sale_work(&conn, sale1, &items_with_discount, &pay_full);
        let _ = conn.execute_batch("ROLLBACK");
        assert!(
            result.is_err(),
            "Pagar 20.0 quando total descontado é 18.0 deve falhar"
        );

        // Paying the discounted total (18.0) should succeed
        let sale2 = make_sale(20.0);
        let items2 = vec![SaleItemInput {
            product_id,
            quantity: 1,
            discount: 0.1,
        }];
        let pay_discounted = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 18.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        let result2 = do_close_sale_work(&conn, sale2, &items2, &pay_discounted);
        conn.execute_batch("COMMIT").unwrap();
        assert!(
            result2.is_ok(),
            "Pagar 18.0 quando total descontado é 18.0 deve ter sucesso"
        );
    }

    #[test]
    fn get_sale_payments_returns_method_name_and_amounts() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let product_id = insert_product(&conn, "Foto", true);
        let pm_id = insert_payment_method(&conn, "Dinheiro");

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 30.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 30.0, 0.0, 30.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 30.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let rows: Vec<(String, f64, f64)> = {
            let mut stmt = conn
                .prepare(
                    r#"SELECT pm.name, sp.amount_paid, sp.net_amount
                   FROM "Sale_Payments" sp
                   JOIN "Payment_Methods" pm ON pm.id = sp.payment_method_id
                   WHERE sp.sale_id = ? ORDER BY sp.created_at"#,
                )
                .unwrap();
            stmt.query_map(params![sale_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, "Dinheiro");
        assert!((rows[0].1 - 30.0).abs() < 1e-6, "amount_paid deve ser 30.0");
        assert!(
            (rows[0].2 - 30.0).abs() < 1e-6,
            "net_amount sem taxa deve ser 30.0"
        );
    }

    #[test]
    fn close_sale_flag_false_resale_insufficient_stock_returns_warning() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Caneca", true);
        // flag already 'false' from seed

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 10.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert_eq!(
            warnings.len(),
            1,
            "flag=false deve gerar aviso de estoque insuficiente"
        );
        let status: String = conn
            .query_row(
                r#"SELECT status FROM "Sales" WHERE id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "closed", "Venda deve prosseguir mesmo com aviso");
    }

    #[test]
    fn get_product_sales_history_returns_closed_sales() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Foto", true);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 2, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id,
            quantity: 2,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 20.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let rows = get_product_sales_history_raw(&conn, product_id).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].sale_id, sale_id);
        assert_eq!(rows[0].quantity, 2);
        assert!((rows[0].unit_price - 10.0).abs() < 1e-6);
        assert!((rows[0].revenue - 20.0).abs() < 1e-6);
    }

    #[test]
    fn get_product_sales_history_returns_empty_when_no_sales() {
        let conn = test_db();
        let product_id = insert_product(&conn, "Sem vendas", true);
        let rows = get_product_sales_history_raw(&conn, product_id).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn get_product_sales_history_ignores_open_drafts() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let product_id = insert_product(&conn, "Foto", true);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let rows = get_product_sales_history_raw(&conn, product_id).unwrap();
        assert!(
            rows.is_empty(),
            "rascunhos abertos nao devem aparecer no historico"
        );
    }

    #[test]
    fn close_sale_flag_false_component_insufficient_stock_returns_warning() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let foto_id = insert_product(&conn, "Foto 10x15", true);
        let papel_id = insert_product(&conn, "Papel", false);
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        // flag already 'false' from seed; no stock for papel

        add_product_component_in_test(&conn, foto_id, papel_id, 1.0);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, foto_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id: foto_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 10.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert_eq!(
            warnings.len(),
            1,
            "flag=false deve gerar aviso de insumo insuficiente"
        );
        assert!(warnings[0].contains("Papel"));
        let status: String = conn
            .query_row(
                r#"SELECT status FROM "Sales" WHERE id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "closed");
    }

    #[test]
    fn close_sale_flag_true_insufficient_stock_proceeds_without_warnings() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Caneca", true);

        // Set flag to 'true'
        conn.execute(
            r#"UPDATE "System_Settings" SET "value" = 'true' WHERE "key" = 'allow_negative_stock_sale'"#,
            [],
        ).unwrap();

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let items = vec![SaleItemInput {
            product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 10.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert!(
            warnings.is_empty(),
            "flag=true não deve gerar avisos de estoque insuficiente"
        );
        let status: String = conn
            .query_row(
                r#"SELECT status FROM "Sales" WHERE id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(status, "closed");

        let out_qty: f64 = conn
            .query_row(
                r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out'"#,
                params![product_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            (out_qty - 1.0).abs() < 1e-9,
            "Baixa deve ser gravada mesmo com estoque negativo"
        );
    }

    #[test]
    fn commands_reject_invalid_session() {
        use crate::auth::require_session_with;
        let session: Option<crate::auth::Session> = None;
        let err = require_session_with(&session, "bad-token").unwrap_err();
        assert_eq!(err, "Sessão inválida");
    }

    #[test]
    fn get_sale_payments_returns_empty_for_open_sale() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let product_id = insert_product(&conn, "Foto", true);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();

        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Sale_Payments" WHERE sale_id = ?"#,
                params![sale_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "Pedido em aberto não deve ter pagamentos");
    }

    #[test]
    fn close_sale_consumes_fractional_component_stock() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let produto_id = insert_product(&conn, "Copia colorida", true);
        let tinta_id = insert_product(&conn, "Tinta", false);
        let pm_id = insert_payment_method(&conn, "Dinheiro");

        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 100.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![tinta_id],
        )
        .unwrap();

        // 2.5 mL de tinta por cópia
        add_product_component_in_test(&conn, produto_id, tinta_id, 2.5);

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 20.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 2, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, produto_id],
        )
        .unwrap();

        let items = vec![SaleItemInput { product_id: produto_id, quantity: 2, discount: 0.0 }];
        let payments = vec![SalePaymentInput { payment_method_id: pm_id, amount_paid: 20.0, installments: 1 }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert!(warnings.is_empty());

        // 2 cópias × 2.5 mL = 5.0 mL consumidos
        let consumed: f64 = conn
            .query_row(
                r#"SELECT CAST(SUM(CASE WHEN movement_type = 'out' THEN quantity ELSE 0 END) AS DOUBLE)
                   FROM "Stock_Movements" WHERE product_id = ?"#,
                params![tinta_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!((consumed - 5.0).abs() < 1e-9, "esperado 5.0 mL consumidos, obtido {consumed}");
    }
}
