use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;

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
pub fn list_sales(status: String, state: State<DbState>) -> Result<Vec<SaleRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT id, label, total_value, CAST(created_at AS VARCHAR)
            FROM "Sales"
            WHERE status = ?
            ORDER BY created_at DESC
        "#)
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
        item_data.push((item.product_id, item.quantity, final_price, discount, discounted_price));
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
            params![sale_id, product_id, quantity, price, discount, discounted_price],
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
    state: State<DbState>,
) -> Result<i64, String> {
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
    state: State<DbState>,
) -> Result<(), String> {
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
                params![id, item.product_id, item.quantity, final_price, discount, discounted_price],
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
    payments: &[SalePaymentInput],
) -> Result<Vec<String>, String> {
    let (status, total_value): (String, f64) = conn
        .query_row(
            r#"SELECT status, total_value FROM "Sales" WHERE id = ?"#,
            params![sale_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| format!("Venda {} não encontrada", sale_id))?;

    if status != "open" {
        return Err("Venda já foi fechada".to_string());
    }

    let total_paid: f64 = payments.iter().map(|p| p.amount_paid).sum();
    if (total_paid - total_value).abs() > 0.01 {
        return Err(format!(
            "Valor pago ({:.2}) não corresponde ao total da venda ({:.2})",
            total_paid, total_value
        ));
    }

    for payment in payments {
        let fee_percent: f64 = conn
            .query_row(
                r#"SELECT COALESCE(MAX(fee_percent), 0.0)
                   FROM "Transaction_Fees"
                   WHERE payment_method_id = ?"#,
                params![payment.payment_method_id],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        let net_amount = payment.amount_paid * (1.0 - fee_percent);

        conn.execute(
            r#"INSERT INTO "Sale_Payments"
               (sale_id, payment_method_id, amount_paid, net_amount, created_at)
               VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)"#,
            params![sale_id, payment.payment_method_id, payment.amount_paid, net_amount],
        )
        .map_err(|e| e.to_string())?;
    }

    let mut stmt = conn
        .prepare(r#"
            SELECT sp.product_id, sp.quantity, p.sellable
            FROM "Sales_Products" sp
            JOIN "Products" p ON p.id = sp.product_id
            WHERE sp.sale_id = ?
        "#)
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map(params![sale_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i32>(1)?, row.get::<_, bool>(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut warnings: Vec<String> = Vec::new();

    for (product_id, quantity, sellable) in items {
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
                .prepare(r#"
                    SELECT pc.child_id, pc.quantity, p.name,
                           COALESCE(s.current_quantity, 0.0)
                    FROM "Product_Components" pc
                    JOIN "Products" p ON p.id = pc.child_id
                    LEFT JOIN "Stock" s ON s.product_id = pc.child_id
                    WHERE pc.parent_id = ?
                "#)
                .map_err(|e| e.to_string())?;

            let components: Vec<(i64, i64, String, f64)> = comp_stmt
                .query_map(params![product_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            for (child_id, comp_qty, child_name, current_stock) in components {
                let qty_to_consume = comp_qty as f64 * quantity as f64;

                if current_stock < qty_to_consume {
                    warnings.push(format!(
                        "Estoque insuficiente de '{}': disponível {:.2}, necessário {:.2}",
                        child_name, current_stock, qty_to_consume
                    ));
                }

                conn.execute(
                    r#"INSERT INTO "Stock_Movements"
                       (product_id, quantity, movement_type, motive, created_at)
                       VALUES (?, ?, 'out', 'venda', CURRENT_TIMESTAMP)"#,
                    params![child_id, qty_to_consume],
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
    payments: Vec<SalePaymentInput>,
    state: State<DbState>,
) -> Result<Vec<String>, String> {
    if payments.is_empty() {
        return Err("Informe ao menos um pagamento".to_string());
    }

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute_batch("BEGIN").map_err(|e| e.to_string())?;

    match do_close_sale_work(&conn, id, &payments) {
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

#[tauri::command]
pub fn get_sale_items(sale_id: i64, state: State<DbState>) -> Result<Vec<SaleItemRow>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT sp.product_id, p.name, sp.quantity, sp.price, sp.discount, sp.discounted_price
            FROM "Sales_Products" sp
            JOIN "Products" p ON p.id = sp.product_id
            WHERE sp.sale_id = ?
            ORDER BY sp.created_at
        "#)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_db;
    use crate::products::insert_product;
    use crate::settings::insert_payment_method;
    use crate::auth::insert_user;
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
        let user_id: i64 = conn.query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0)).unwrap();
        let product_id = insert_product(&conn, "Foto 10x15", true);

        conn.execute_batch("BEGIN").unwrap();
        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
                   VALUES (?, 'João', 'open', 25.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            ).unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products"
               (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 2, 15.0, 0.1, 13.5, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        ).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Sales" WHERE label = 'João'"#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
        let item_count: i64 = conn
            .query_row(r#"SELECT COUNT(*) FROM "Sales_Products" WHERE sale_id = ?"#, params![sale_id], |r| r.get(0))
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
        let user_id: i64 = conn.query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0)).unwrap();
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
            ).unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products"
               (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 2, 15.0, 0.0, 15.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, sellable_id],
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products"
               (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 0.0, 0.0, 0.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, insumo_id],
        ).unwrap();

        let payments = vec![SalePaymentInput { payment_method_id: pm_id, amount_paid: 30.0 }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        let prod_count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "Production_Orders" WHERE sale_id = ?"#,
                params![sale_id], |r| r.get(0),
            ).unwrap();
        assert_eq!(prod_count, 1);

        let status: String = conn
            .query_row(r#"SELECT status FROM "Sales" WHERE id = ?"#, params![sale_id], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "closed");
    }

    #[test]
    fn close_sale_consumes_stock_for_components() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn.query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0)).unwrap();
        let foto_id = insert_product(&conn, "Foto 10x15", true);
        let papel_id = insert_product(&conn, "Papel", false);
        let pm_id = insert_payment_method(&conn, "Dinheiro");

        conn.execute(
            r#"INSERT INTO "Stock_Movements" (product_id, quantity, movement_type, motive, created_at)
               VALUES (?, 100.0, 'in', 'compra', CURRENT_TIMESTAMP)"#,
            params![papel_id],
        ).unwrap();

        add_product_component_in_test(&conn, foto_id, papel_id, 2);

        let sale_id: i64 = conn.query_row(
            r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 30.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
            params![user_id],
            |r| r.get(0),
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 3, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, foto_id],
        ).unwrap();

        let payments = vec![SalePaymentInput { payment_method_id: pm_id, amount_paid: 30.0 }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert!(warnings.is_empty(), "Não deveria haver avisos com estoque suficiente");

        let consumed: f64 = conn.query_row(
            r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out'"#,
            params![papel_id],
            |r| r.get(0),
        ).unwrap();
        assert!((consumed - 6.0).abs() < 1e-9, "Esperado consumo de 6 folhas, foi {}", consumed);

        let input_count: i64 = conn.query_row(
            r#"SELECT COUNT(*) FROM "Production_Inputs" WHERE product_id = ?"#,
            params![papel_id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(input_count, 1);
    }

    #[test]
    fn close_sale_with_insufficient_stock_returns_warning_and_proceeds() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn.query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0)).unwrap();
        let foto_id = insert_product(&conn, "Foto 10x15", true);
        let papel_id = insert_product(&conn, "Papel", false);
        let pm_id = insert_payment_method(&conn, "Dinheiro");

        add_product_component_in_test(&conn, foto_id, papel_id, 1);

        let sale_id: i64 = conn.query_row(
            r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
               VALUES (?, '', 'open', 10.0, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
               RETURNING id"#,
            params![user_id],
            |r| r.get(0),
        ).unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 10.0, 0.0, 10.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, foto_id],
        ).unwrap();

        let payments = vec![SalePaymentInput { payment_method_id: pm_id, amount_paid: 10.0 }];
        conn.execute_batch("BEGIN").unwrap();
        let warnings = do_close_sale_work(&conn, sale_id, &payments).unwrap();
        conn.execute_batch("COMMIT").unwrap();

        assert_eq!(warnings.len(), 1, "Deveria haver 1 aviso de estoque insuficiente");
        assert!(warnings[0].contains("Papel"));

        let status: String = conn
            .query_row(r#"SELECT status FROM "Sales" WHERE id = ?"#, params![sale_id], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "closed");

        let consumed: f64 = conn.query_row(
            r#"SELECT COALESCE(SUM(quantity), 0.0) FROM "Stock_Movements"
               WHERE product_id = ? AND movement_type = 'out'"#,
            params![papel_id],
            |r| r.get(0),
        ).unwrap();
        assert!((consumed - 1.0).abs() < 1e-9);
    }
}
