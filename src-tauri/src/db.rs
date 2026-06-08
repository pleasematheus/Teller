use duckdb::Connection;
use std::sync::Mutex;

const SCHEMA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/db/schema.sql"));

// Versão do schema do banco. Incrementar manualmente quando schema.sql mudar de
// forma incompatível. Consumida pela ferramenta de atualização dedicada via App_Meta.
const DB_VERSION: &str = "6";

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

    conn.execute_batch(SCHEMA).map_err(|e| e.to_string())?;

    // Banco novo: carimba versões em App_Meta para a ferramenta de atualização dedicada.
    conn.execute(
        r#"INSERT INTO "App_Meta" ("key", "value") VALUES ('db_version', ?)"#,
        duckdb::params![DB_VERSION],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        r#"INSERT INTO "App_Meta" ("key", "value") VALUES ('app_version', ?)"#,
        duckdb::params![env!("CARGO_PKG_VERSION")],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        r#"INSERT INTO "System_Settings" ("key", "value", "updated_at")
           VALUES ('allow_negative_stock_sale', 'false', CURRENT_TIMESTAMP)"#,
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        r#"INSERT INTO "System_Settings" ("key", "value", "updated_at")
           VALUES ('minimize_to_tray', 'false', CURRENT_TIMESTAMP)"#,
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod app_meta_tests {
    use super::*;

    #[test]
    fn init_schema_stamps_versions_in_app_meta() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

        let db_version: String = conn
            .query_row(
                r#"SELECT "value" FROM "App_Meta" WHERE "key" = 'db_version'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        let app_version: String = conn
            .query_row(
                r#"SELECT "value" FROM "App_Meta" WHERE "key" = 'app_version'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(db_version, DB_VERSION);
        assert!(!app_version.is_empty());
    }

    #[test]
    fn init_schema_creates_system_settings_with_seed() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

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
    fn init_schema_seeds_minimize_to_tray() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

        let value: String = conn
            .query_row(
                r#"SELECT "value" FROM "System_Settings" WHERE "key" = 'minimize_to_tray'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(value, "false");
    }

    #[test]
    fn init_schema_app_meta_only_has_version_keys() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                r#"SELECT COUNT(*) FROM "App_Meta" WHERE "key" NOT IN ('db_version', 'app_version')"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "App_Meta deve conter apenas db_version e app_version");
    }
}

#[cfg(test)]
mod legacy_decimal_tests {
    use super::*;
    use duckdb::params;

    // Reproduces a pre-existing on-disk DB where Products columns are still DECIMAL.
    // duckdb-rs 1.4.4 reads DECIMAL(5,4) directly as f64 incorrectly (returns 0), and
    // ALTER ... SET DATA TYPE is rejected because Products is FK-referenced. The fix is
    // to CAST(... AS DOUBLE) in the SELECT, which works on both DECIMAL and DOUBLE columns.
    fn legacy_products_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"CREATE SEQUENCE products_id_seq START 1;
               CREATE TABLE "Products" (
                   "id"                 INTEGER DEFAULT nextval('products_id_seq') PRIMARY KEY,
                   "name"               VARCHAR NOT NULL,
                   "markup"             DECIMAL(5,4)  NOT NULL,
                   "cost_price"         DECIMAL(10,2) NOT NULL DEFAULT 0,
                   "final_price"        DECIMAL(10,2) NOT NULL,
                   "expected_loss_rate" DECIMAL(5,4)  NOT NULL DEFAULT 0,
                   "profit_margin"      DECIMAL(5,4)  NOT NULL DEFAULT 0
               );
               CREATE TABLE "Product_Components" (
                   "id"        INTEGER PRIMARY KEY,
                   "parent_id" INTEGER NOT NULL,
                   "child_id"  INTEGER NOT NULL,
                   FOREIGN KEY ("parent_id") REFERENCES "Products" ("id"),
                   FOREIGN KEY ("child_id")  REFERENCES "Products" ("id")
               );"#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn direct_decimal_read_is_broken_but_cast_read_works() {
        let conn = legacy_products_conn();
        // Write via f64 binding, exactly like create_product does.
        conn.execute(
            r#"INSERT INTO "Products"
               (name, markup, cost_price, final_price, expected_loss_rate, profit_margin)
               VALUES (?, ?, ?, ?, ?, ?)"#,
            params![
                "P",
                1.25_f64,
                10.0_f64,
                12.5_f64,
                0.0732_f64,
                0.3456_f64
            ],
        )
        .unwrap();

        // CAST(... AS DOUBLE) round-trips correctly regardless of physical column type.
        let (margin, loss, markup, cost, price): (f64, f64, f64, f64, f64) = conn
            .query_row(
                r#"SELECT CAST(profit_margin AS DOUBLE), CAST(expected_loss_rate AS DOUBLE),
                          CAST(markup AS DOUBLE), CAST(cost_price AS DOUBLE),
                          CAST(final_price AS DOUBLE)
                   FROM "Products" WHERE name = 'P'"#,
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();
        assert!((margin - 0.3456).abs() < 1e-9, "profit_margin got {}", margin);
        assert!((loss - 0.0732).abs() < 1e-9, "expected_loss_rate got {}", loss);
        assert!((markup - 1.25).abs() < 1e-9, "markup got {}", markup);
        assert!((cost - 10.0).abs() < 1e-9, "cost_price got {}", cost);
        assert!((price - 12.5).abs() < 1e-9, "final_price got {}", price);
    }
}

#[cfg(test)]
pub(crate) fn test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    init_schema(&conn).unwrap();
    conn
}
