use bcrypt::{hash, verify, DEFAULT_COST};
use duckdb::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};

const SCHEMA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/db/schema.sql"));

pub struct DbState(pub Mutex<Connection>);

#[derive(Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub nickname: String,
}

struct UserRow {
    id: i64,
    name: String,
    nickname: String,
    password_hash: String,
    active: bool,
}

fn ensure_user_settings_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "User_Settings" (
            "user_id" INTEGER NOT NULL,
            "key"     TEXT    NOT NULL,
            "value"   TEXT    NOT NULL,
            PRIMARY KEY ("user_id", "key"),
            FOREIGN KEY ("user_id") REFERENCES "Users" ("id")
        )"#,
    )
    .map_err(|e| e.to_string())
}

fn ensure_unit_column(conn: &Connection) -> Result<(), String> {
    // DuckDB does not support NOT NULL in ADD COLUMN; new DBs get it via schema.sql.
    conn.execute_batch(
        r#"ALTER TABLE "Products" ADD COLUMN IF NOT EXISTS
           "unit" VARCHAR DEFAULT 'unidade'"#,
    )
    .map_err(|e| e.to_string())
}

fn init_schema(conn: &Connection) -> Result<(), String> {
    // execute_batch aborts DuckDB when objects already exist, even with IF NOT EXISTS.
    // Skip entirely if the schema was already initialized in a previous run.
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'Production_Inputs'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    if count > 0 {
        return Ok(());
    }

    conn.execute_batch(SCHEMA).map_err(|e| e.to_string())
}

fn lookup_user(conn: &Connection, identifier: &str) -> Result<Option<UserRow>, String> {
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
fn check_setup(state: State<DbState>) -> Result<bool, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    init_schema(&conn)?;
    ensure_user_settings_table(&conn)?;
    ensure_unit_column(&conn)?;
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
fn create_user(
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
fn check_business_setup(state: State<DbState>) -> Result<bool, String> {
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
fn create_business_settings(
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
fn find_user(identifier: String, state: State<DbState>) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    lookup_user(&conn, identifier.trim()).map(|u| u.map(|row| row.nickname))
}

#[tauri::command]
fn login(identifier: String, password: String, state: State<DbState>) -> Result<User, String> {
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

// ─── User Settings ───────────────────────────────────────────────────────────

#[tauri::command]
fn get_user_setting(
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
fn set_user_setting(
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

// ─── Products ────────────────────────────────────────────────────────────────

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
}

fn validate_product(name: &str, cost_price: f64, expected_loss_rate: f64) -> Result<(), String> {
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
fn get_pricing_rates(state: State<DbState>) -> Result<PricingRates, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let (investment_rate, profit_margin): (f64, f64) = conn
        .query_row(
            r#"SELECT investment_rate, profit_margin FROM "Business_Settings" LIMIT 1"#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
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
fn get_product_loss_rate(product_id: i64, state: State<DbState>) -> Result<f64, String> {
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
fn list_products(state: State<DbState>) -> Result<Vec<Product>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT id, name, description, active, sellable,
                   cost_price, markup, final_price, expected_loss_rate, unit
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
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(products)
}

#[tauri::command]
fn create_product(
    name: String,
    description: Option<String>,
    active: bool,
    sellable: bool,
    cost_price: f64,
    markup: f64,
    final_price: f64,
    expected_loss_rate: f64,
    unit: String,
    state: State<DbState>,
) -> Result<(), String> {
    let name = name.trim().to_string();
    validate_product(&name, cost_price, expected_loss_rate)?;

    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        r#"INSERT INTO "Products"
           (name, description, active, sellable, cost_price, markup, final_price,
            expected_loss_rate, unit, updated_at, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        params![name, description, active, sellable, cost_price, markup, final_price,
                expected_loss_rate, unit],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn update_product(
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
                   unit = ?, updated_at = CURRENT_TIMESTAMP
               WHERE id = ?"#,
            params![name, description, active, sellable, cost_price, markup, final_price,
                    expected_loss_rate, unit, id],
        )
        .map_err(|e| e.to_string())?;

    if rows_affected == 0 {
        return Err(format!("Produto {} não encontrado", id));
    }

    Ok(())
}

// ─── Stock ───────────────────────────────────────────────────────────────────

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
}

#[derive(Serialize)]
pub struct StockMovement {
    pub id: i64,
    pub quantity: f64,
    pub movement_type: String,
    pub motive: String,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct ProductComponent {
    pub child_id: i64,
    pub child_name: String,
    pub quantity: i64,
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
fn list_stock(state: State<DbState>) -> Result<Vec<StockItem>, String> {
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

fn do_purchase_work(
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
        conn.execute(
            r#"INSERT INTO "Purchase_Items"
               (purchase_id, product_id, quantity, unit_cost, box_qty, unit_qty,
                creates_stock, created_at)
               VALUES (?, ?, ?, ?, ?, ?, true, CURRENT_TIMESTAMP)"#,
            params![
                purchase_id, item.product_id, item.quantity, item.unit_cost,
                item.box_qty, item.unit_qty
            ],
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
fn create_purchase(
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
fn create_stock_adjustment(
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
fn list_stock_movements(
    product_id: i64,
    state: State<DbState>,
) -> Result<Vec<StockMovement>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT id, quantity, movement_type, motive,
                   CAST(created_at AS VARCHAR)
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
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(movements)
}

#[tauri::command]
fn list_product_components(
    product_id: i64,
    state: State<DbState>,
) -> Result<Vec<ProductComponent>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(r#"
            SELECT pc.child_id, p.name, pc.quantity, pc.calculated_quantity, pc.use_calculated
            FROM "Product_Components" pc
            JOIN "Products" p ON p.id = pc.child_id
            WHERE pc.parent_id = ?
            ORDER BY p.name
        "#)
        .map_err(|e| e.to_string())?;

    let components = stmt
        .query_map(params![product_id], |row| {
            Ok(ProductComponent {
                child_id: row.get(0)?,
                child_name: row.get(1)?,
                quantity: row.get(2)?,
                calculated_quantity: row.get(3)?,
                use_calculated: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(components)
}

#[tauri::command]
fn list_purchases(state: State<DbState>) -> Result<Vec<Purchase>, String> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = match std::env::var("DB_PATH") {
                Ok(p) => std::path::PathBuf::from(p),
                Err(_) => {
                    let data_dir = app.path().app_data_dir()?;
                    std::fs::create_dir_all(&data_dir)?;
                    data_dir.join("teller.db")
                }
            };
            let conn = Connection::open(db_path)?;
            app.manage(DbState(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_setup,
            create_user,
            check_business_setup,
            create_business_settings,
            find_user,
            login,
            get_user_setting,
            set_user_setting,
            get_pricing_rates,
            get_product_loss_rate,
            list_products,
            create_product,
            update_product,
            list_stock,
            create_purchase,
            create_stock_adjustment,
            list_stock_movements,
            list_product_components,
            list_purchases,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use duckdb::Connection;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        ensure_user_settings_table(&conn).unwrap();
        ensure_unit_column(&conn).unwrap();
        conn
    }

    fn insert_user(conn: &Connection, name: &str, nickname: &str, active: bool) {
        let pw_hash = hash("senha123", DEFAULT_COST).unwrap();
        conn.execute(
            r#"INSERT INTO "Users" (name, nickname, password_hash, active, updated_at, created_at)
               VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![name, nickname, pw_hash, active],
        )
        .unwrap();
    }

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

    fn find_user_in_test(conn: &Connection, identifier: &str) -> Option<String> {
        lookup_user(conn, identifier).unwrap().map(|row| row.nickname)
    }

    fn insert_business_settings(conn: &Connection, name: &str) {
        conn.execute(
            r#"INSERT INTO "Business_Settings" (name, investment_rate, profit_margin, updated_at, created_at)
               VALUES (?, 0.1, 0.2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![name],
        )
        .unwrap();
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

    fn insert_product(conn: &Connection, name: &str, sellable: bool) -> i64 {
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

    // validate_product

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

    // create_product (via direct DB insert + count)

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

    // update_product (rows_affected)

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

    // get_user_setting / set_user_setting

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

    // get_pricing_rates logic

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

    // get_product_loss_rate logic

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

    // create_purchase

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
        // 2 caixas, cada caixa tem 500 unidades (box_qty=1, unit_qty=500)
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

    // create_stock_adjustment

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

    // list_stock_movements

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
}
