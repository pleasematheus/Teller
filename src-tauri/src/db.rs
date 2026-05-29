use duckdb::Connection;
use std::sync::Mutex;

const SCHEMA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/db/schema.sql"));

pub fn open_database(path: &std::path::Path) -> Result<Connection, duckdb::Error> {
    match Connection::open(path) {
        Ok(conn) => Ok(conn),
        Err(_) => {
            // WAL replay can fail due to a DuckDB 1.4.4 bug ("DatabaseManager::GetDefaultDatabase
            // with no default database set"). Delete the stale WAL and retry; the main DB file
            // remains consistent since WAL data hasn't been applied yet.
            let wal_path = format!("{}.wal", path.display());
            let _ = std::fs::remove_file(&wal_path);
            Connection::open(path)
        }
    }
}

pub struct DbState(pub Mutex<Connection>);

pub fn ensure_user_settings_table(conn: &Connection) -> Result<(), String> {
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

pub fn ensure_unit_column(conn: &Connection) -> Result<(), String> {
    // DuckDB does not support NOT NULL in ADD COLUMN; new DBs get it via schema.sql.
    conn.execute_batch(
        r#"ALTER TABLE "Products" ADD COLUMN IF NOT EXISTS
           "unit" VARCHAR DEFAULT 'unidade'"#,
    )
    .map_err(|e| e.to_string())
}

pub fn ensure_profit_margin_column(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"ALTER TABLE "Products" ADD COLUMN IF NOT EXISTS
           "profit_margin" DECIMAL(5,4) DEFAULT 0"#,
    )
    .map_err(|e| e.to_string())
}

pub fn ensure_label_column(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"ALTER TABLE "Sales" ADD COLUMN IF NOT EXISTS
           "label" VARCHAR DEFAULT ''"#,
    )
    .map_err(|e| e.to_string())
}

pub fn ensure_sales_products_cost_price(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"ALTER TABLE "Sales_Products" ADD COLUMN IF NOT EXISTS
           "cost_price" DECIMAL(10,2)"#,
    )
    .map_err(|e| e.to_string())
}

pub fn ensure_financial_entries_expense_subtype(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"ALTER TABLE "Financial_Entries" ADD COLUMN IF NOT EXISTS
           "expense_subtype" VARCHAR"#,
    )
    .map_err(|e| e.to_string())
}

pub fn ensure_business_settings_nullable(conn: &Connection) -> Result<(), String> {
    // Idempotent: ignore errors (DuckDB may error if column is already nullable).
    let _ = conn.execute_batch(
        r#"ALTER TABLE "Business_Settings" ALTER COLUMN investment_rate DROP NOT NULL;
           ALTER TABLE "Business_Settings" ALTER COLUMN profit_margin DROP NOT NULL;"#,
    );
    Ok(())
}

pub fn ensure_business_settings_double_type(conn: &Connection) -> Result<(), String> {
    // duckdb-rs 1.4.4 cannot read DECIMAL(5,4) as f64 correctly (scale not applied).
    // Migrate to DOUBLE which maps directly to f64. Idempotent: ignore errors.
    let _ = conn.execute_batch(
        r#"ALTER TABLE "Business_Settings" ALTER COLUMN investment_rate SET DATA TYPE DOUBLE;
           ALTER TABLE "Business_Settings" ALTER COLUMN profit_margin SET DATA TYPE DOUBLE;"#,
    );
    Ok(())
}

pub fn ensure_transaction_fees_installments(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"ALTER TABLE "Transaction_Fees" ADD COLUMN IF NOT EXISTS
           "installments" INTEGER DEFAULT 1"#,
    )
    .map_err(|e| e.to_string())?;
    // Make description nullable — ignore errors (may already be nullable or unsupported).
    let _ = conn.execute_batch(
        r#"ALTER TABLE "Transaction_Fees" ALTER COLUMN description DROP NOT NULL"#,
    );
    // duckdb-rs 1.4.4 cannot read DECIMAL(5,4) as f64 correctly; migrate to DOUBLE.
    let _ = conn.execute_batch(
        r#"ALTER TABLE "Transaction_Fees" ALTER COLUMN fee_percent SET DATA TYPE DOUBLE;
           ALTER TABLE "Transaction_Fees" ALTER COLUMN fixed_fee SET DATA TYPE DOUBLE"#,
    );
    Ok(())
}

pub fn ensure_sale_payments_fee_snapshot(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"ALTER TABLE "Sale_Payments" ADD COLUMN IF NOT EXISTS
           "applied_fee_percent" DOUBLE DEFAULT 0"#,
    )
    .map_err(|e| e.to_string())?;
    conn.execute_batch(
        r#"ALTER TABLE "Sale_Payments" ADD COLUMN IF NOT EXISTS
           "applied_fixed_fee" DOUBLE DEFAULT 0"#,
    )
    .map_err(|e| e.to_string())?;
    // net_amount: migrate from DECIMAL(10,2) to DOUBLE to avoid duckdb-rs scale rounding.
    let _ = conn.execute_batch(
        r#"ALTER TABLE "Sale_Payments" ALTER COLUMN net_amount SET DATA TYPE DOUBLE"#,
    );
    Ok(())
}

fn parse_installments_from_description(description: &str) -> Option<i64> {
    // Match descriptions like "2x", "12x", "3 x" (case-insensitive, trailing x).
    let lower = description.trim().to_lowercase();
    if lower.ends_with('x') {
        let num_part = lower[..lower.len() - 1].trim();
        num_part.parse::<i64>().ok().filter(|&n| n >= 1)
    } else {
        None
    }
}

pub fn normalize_transaction_fee_installments(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            r#"SELECT id, description FROM "Transaction_Fees"
               WHERE installments = 1 AND description IS NOT NULL"#,
        )
        .map_err(|e| e.to_string())?;

    let rows: Vec<(i64, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (id, description) in rows {
        let installments = parse_installments_from_description(&description).unwrap_or(1);
        if installments > 1 {
            conn.execute(
                r#"UPDATE "Transaction_Fees" SET installments = ? WHERE id = ?"#,
                duckdb::params![installments, id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

pub fn init_schema(conn: &Connection) -> Result<(), String> {
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

#[cfg(test)]
pub(crate) fn test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    init_schema(&conn).unwrap();
    ensure_user_settings_table(&conn).unwrap();
    ensure_unit_column(&conn).unwrap();
    ensure_profit_margin_column(&conn).unwrap();
    ensure_label_column(&conn).unwrap();
    ensure_sales_products_cost_price(&conn).unwrap();
    ensure_financial_entries_expense_subtype(&conn).unwrap();
    // Migration functions are no-ops for fresh in-memory DBs (schema.sql already has all columns).
    // They run in production via lib.rs setup for existing on-disk databases.
    conn
}
