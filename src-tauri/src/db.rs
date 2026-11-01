use duckdb::Connection;
use std::sync::Mutex;

const SCHEMA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/db/schema.sql"));

// Versão do schema. Incrementar em lockstep com novo arquivo em src/db/migrations/ a cada
// mudança incompatível de schema.sql. Consumida pelo runner de migração embutido.
const DB_VERSION: &str = "9";

pub fn open_database(path: &std::path::Path) -> Result<Connection, duckdb::Error> {
    // Deixe o DuckDB recuperar o WAL. Uma falha de abertura não autoriza sua exclusão.
    Connection::open(path)
}

pub fn initialize_database(path: &std::path::Path) -> Result<Connection, String> {
    let conn = open_database(path).map_err(|e| e.to_string())?;
    init_schema(&conn)?;
    drop(conn);
    run_migrations(Some(path))?;
    let conn = open_database(path).map_err(|e| e.to_string())?;
    conn.execute(
        r#"UPDATE "App_Meta" SET "value" = ? WHERE "key" = 'app_version'"#,
        duckdb::params![env!("CARGO_PKG_VERSION")],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn)
}

pub struct DbState(pub Mutex<Connection>);

// --- Migration infrastructure ---

// Payload of a single migration step. Each entry targets exactly one version.
#[allow(dead_code)]
enum MigrationKind {
    Sql(&'static str),
    Rust(fn(&Connection) -> Result<(), String>),
}

struct MigrationStep {
    version: u32,
    kind: MigrationKind,
}

fn migrate_v8(conn: &Connection) -> Result<(), String> {
    // DuckDB 1.4.4 does not support ADD COLUMN with constraints; add nullable first, then backfill.
    conn.execute_batch(r#"ALTER TABLE "Payment_Methods" ADD COLUMN "is_cash" BOOLEAN;"#)
        .map_err(|e| e.to_string())?;
    conn.execute(r#"UPDATE "Payment_Methods" SET "is_cash" = false"#, [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn migrate_v9(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"ALTER TABLE "Product_Components" ALTER COLUMN "quantity" TYPE DECIMAL(10,4);"#,
    )
    .map_err(|e| e.to_string())
}

// Append-only. Baseline is v6; first real entry targets v7.
// Never edit or remove released entries — only append new ones.
static MIGRATIONS: &[MigrationStep] = &[
    MigrationStep {
        version: 8,
        kind: MigrationKind::Rust(migrate_v8),
    },
    MigrationStep {
        version: 9,
        kind: MigrationKind::Rust(migrate_v9),
    },
];

fn read_db_version(conn: &Connection) -> Result<u32, String> {
    conn.query_row(
        r#"SELECT "value" FROM "App_Meta" WHERE "key" = 'db_version'"#,
        [],
        |r| r.get::<_, String>(0),
    )
    .map_err(|_| "db_version ausente em App_Meta (banco anômalo)".to_string())
    .and_then(|v| {
        v.parse::<u32>()
            .map_err(|e| format!("db_version inválido: {e}"))
    })
}

fn backup_db_file(path: &std::path::Path, stored: u32) -> Result<std::path::PathBuf, String> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".bak.{stored}.{ts}"));
    let backup = std::path::PathBuf::from(name);
    std::fs::copy(path, &backup)
        .map_err(|e| format!("Falha ao criar backup em {}: {e}", backup.display()))?;
    Ok(backup)
}

fn adjacent_path(path: &std::path::Path, suffix: &str) -> std::path::PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    std::path::PathBuf::from(name)
}

fn ensure_no_wal(db: &std::path::Path) -> Result<(), String> {
    let wal = adjacent_path(db, ".wal");
    match wal.try_exists() {
        Ok(false) => Ok(()),
        Ok(true) => Err(format!(
            "Restauração automática recusada: existe um WAL em {}. Preserve o banco, o WAL e o backup para recuperação.",
            wal.display()
        )),
        Err(e) => Err(format!("Falha ao verificar o WAL em {}: {e}", wal.display())),
    }
}

fn restore_backup(backup: &std::path::Path, db: &std::path::Path) -> Result<(), String> {
    // Não associe um backup antigo ao WAL de uma migração incompleta.
    ensure_no_wal(db)?;
    let mut source = std::fs::File::open(backup)
        .map_err(|e| format!("Falha ao ler o backup em {}: {e}", backup.display()))?;
    let staged_path = adjacent_path(db, ".restore");
    let mut staged = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged_path)
        .map_err(|e| {
            format!(
                "Falha ao preparar a restauração em {}: {e}",
                staged_path.display()
            )
        })?;

    // Copie e sincronize antes de substituir o banco. Não sobrescreva um arquivo de recuperação existente.
    let result = (|| {
        std::io::copy(&mut source, &mut staged).map_err(|e| e.to_string())?;
        staged.sync_all().map_err(|e| e.to_string())?;
        drop(staged);
        ensure_no_wal(db)?;
        std::fs::rename(&staged_path, db).map_err(|e| e.to_string())
    })();

    if result.is_err() {
        // Este arquivo foi criado nesta chamada e contém somente a cópia do backup.
        let _ = std::fs::remove_file(&staged_path);
    }
    result.map_err(|e| {
        format!(
            "Falha ao restaurar o backup {} em {}: {e}. O backup foi mantido.",
            backup.display(),
            db.display()
        )
    })
}

fn apply_sql_step(conn: &Connection, sql: &str, version: u32) -> Result<(), String> {
    let batch = format!(
        "BEGIN TRANSACTION;\n{sql}\nUPDATE \"App_Meta\" SET \"value\" = '{version}' WHERE \"key\" = 'db_version';\nCOMMIT;"
    );
    conn.execute_batch(&batch)
        .map_err(|e| format!("Migração v{version} falhou: {e}"))
}

fn apply_rust_step(
    conn: &Connection,
    f: fn(&Connection) -> Result<(), String>,
    version: u32,
) -> Result<(), String> {
    f(conn).map_err(|e| format!("Migração Rust v{version} falhou: {e}"))?;
    conn.execute(
        r#"UPDATE "App_Meta" SET "value" = ? WHERE "key" = 'db_version'"#,
        duckdb::params![version.to_string()],
    )
    .map_err(|e| format!("Carimbo de versão v{version} falhou: {e}"))
    .map(|_| ())
}

fn run_migrations_inner(
    conn: &Connection,
    migrations: &[MigrationStep],
    target: u32,
) -> Result<(), String> {
    let stored = read_db_version(conn)?;

    if stored == target {
        return Ok(());
    }
    if stored > target {
        return Err(format!(
            "Downgrade recusado: banco na versão {stored}, binário suporta até v{target}. Use uma versão mais recente."
        ));
    }

    for step in migrations
        .iter()
        .filter(|s| s.version > stored && s.version <= target)
    {
        match &step.kind {
            MigrationKind::Sql(sql) => apply_sql_step(conn, sql, step.version)?,
            MigrationKind::Rust(f) => apply_rust_step(conn, *f, step.version)?,
        }
    }

    // Verify target was reached; catches missing steps in the chain.
    let reached = read_db_version(conn)?;
    if reached != target {
        return Err(format!(
            "Migração incompleta: banco ficou na versão {reached}, alvo era v{target}. Passos de migração ausentes."
        ));
    }
    Ok(())
}

/// Migra o banco no caminho `db_path` para `DB_VERSION`. Abre e fecha suas próprias conexões
/// para que o backup seja restaurado com segurança antes da conexão principal ser criada.
/// Passa `None` apenas para bancos in-memory (sem arquivo, sem migração).
pub fn run_migrations(db_path: Option<&std::path::Path>) -> Result<(), String> {
    let path = match db_path {
        None => return Ok(()),
        Some(p) => p,
    };
    let target: u32 = DB_VERSION.parse().expect("DB_VERSION must be a valid u32");
    migrate_database(path, MIGRATIONS, target)
}

fn migrate_database(
    path: &std::path::Path,
    migrations: &[MigrationStep],
    target: u32,
) -> Result<(), String> {
    // Phase 1: read version; conn dropped before backup copy to release Windows file lock.
    let backup = {
        let stored = {
            let conn = Connection::open(path).map_err(|e| e.to_string())?;
            let stored = read_db_version(&conn)?;
            if stored == target {
                return Ok(());
            }
            if stored > target {
                return Err(format!(
                    "Downgrade recusado: banco na versão {stored}, binário suporta até v{target}. Use uma versão mais recente."
                ));
            }
            stored
            // conn dropped here: releases file lock before copy
        };
        ensure_no_wal(path)?;
        backup_db_file(path, stored)?
    };

    // Phase 2: migrate (fresh connection, dropped before potential restore).
    let result = Connection::open(path)
        .map_err(|e| e.to_string())
        .and_then(|conn| {
            run_migrations_inner(&conn, migrations, target)
            // conn dropped: DuckDB checkpoints migration state
        });

    if let Err(e) = result {
        return match restore_backup(&backup, path) {
            Ok(()) => Err(format!("{e}. O backup foi restaurado.")),
            Err(restore_error) => Err(format!(
                "{e}. {restore_error} Backup disponível em {}.",
                backup.display()
            )),
        };
    }
    Ok(())
}

// --- Schema initialization ---

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

    // Banco novo: carimba versões em App_Meta para o runner de migração embutido.
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
mod recovery_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "teller_recovery_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn file(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn run_child_test(name: &str, db: &Path) {
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", name, "--nocapture"])
            .env("TELLER_WAL_TEST_PATH", db)
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "processo auxiliar falhou: {}",
            String::from_utf8_lossy(&child.stderr)
        );
    }

    // O DuckDB pode manter um arquivo inválido bloqueado até o processo terminar.
    #[test]
    #[ignore = "processo auxiliar do teste de falha de abertura"]
    fn failed_open_process() {
        let db = std::env::var_os("TELLER_WAL_TEST_PATH").expect("caminho do banco de teste");
        assert!(open_database(Path::new(&db)).is_err());
    }

    #[test]
    fn failed_open_preserves_database_and_wal_bytes() {
        let dir = TestDir::new();
        let db = dir.file("teller.db");
        let wal = adjacent_path(&db, ".wal");
        std::fs::write(&db, b"banco invalido para simular falha").unwrap();
        std::fs::write(&wal, b"WAL que deve ser preservado").unwrap();

        run_child_test("db::recovery_tests::failed_open_process", &db);

        assert_eq!(
            std::fs::read(&db).unwrap(),
            b"banco invalido para simular falha"
        );
        assert_eq!(std::fs::read(&wal).unwrap(), b"WAL que deve ser preservado");
    }

    #[test]
    fn restore_refuses_pending_wal_without_changing_any_file() {
        let dir = TestDir::new();
        let db = dir.file("teller.db");
        let backup = dir.file("backup");
        let wal = adjacent_path(&db, ".wal");
        std::fs::write(&db, b"banco atual").unwrap();
        std::fs::write(&backup, b"backup original").unwrap();
        std::fs::write(&wal, b"transacoes pendentes").unwrap();

        let error = restore_backup(&backup, &db).unwrap_err();

        assert!(error.contains("WAL"));
        assert_eq!(std::fs::read(&db).unwrap(), b"banco atual");
        assert_eq!(std::fs::read(&backup).unwrap(), b"backup original");
        assert_eq!(std::fs::read(&wal).unwrap(), b"transacoes pendentes");
    }

    #[test]
    fn missing_backup_returns_error_and_preserves_database() {
        let dir = TestDir::new();
        let db = dir.file("teller.db");
        std::fs::write(&db, b"banco atual").unwrap();

        assert!(restore_backup(&dir.file("ausente"), &db).is_err());

        assert_eq!(std::fs::read(&db).unwrap(), b"banco atual");
        assert!(!adjacent_path(&db, ".restore").exists());
    }

    #[test]
    fn restore_does_not_overwrite_existing_recovery_file() {
        let dir = TestDir::new();
        let db = dir.file("teller.db");
        let backup = dir.file("backup");
        let staged = adjacent_path(&db, ".restore");
        std::fs::write(&db, b"banco atual").unwrap();
        std::fs::write(&backup, b"backup original").unwrap();
        std::fs::write(&staged, b"recuperacao anterior").unwrap();

        assert!(restore_backup(&backup, &db).is_err());

        assert_eq!(std::fs::read(&db).unwrap(), b"banco atual");
        assert_eq!(std::fs::read(&staged).unwrap(), b"recuperacao anterior");
    }

    #[test]
    fn replacement_failure_returns_error_and_keeps_backup() {
        let dir = TestDir::new();
        let db = dir.file("diretorio");
        let backup = dir.file("backup");
        std::fs::create_dir(&db).unwrap();
        std::fs::write(&backup, b"backup original").unwrap();

        assert!(restore_backup(&backup, &db).is_err());

        assert!(db.is_dir());
        assert_eq!(std::fs::read(&backup).unwrap(), b"backup original");
        assert!(!adjacent_path(&db, ".restore").exists());
    }

    #[test]
    fn failed_migration_restores_original_version_and_data() {
        let dir = TestDir::new();
        let db = dir.file("teller.db");
        {
            let conn = open_database(&db).unwrap();
            conn.execute_batch(
                r#"CREATE TABLE "App_Meta" ("key" VARCHAR PRIMARY KEY, "value" VARCHAR);
                   INSERT INTO "App_Meta" VALUES ('db_version', '10');
                   CREATE TABLE "Sentinel" (v INTEGER);
                   INSERT INTO "Sentinel" VALUES (42);"#,
            )
            .unwrap();
        }
        fn change_then_fail(conn: &Connection) -> Result<(), String> {
            conn.execute_batch(r#"UPDATE "Sentinel" SET v = 99;"#)
                .unwrap();
            Err("falha sintética".to_string())
        }
        let steps = [MigrationStep {
            version: 11,
            kind: MigrationKind::Rust(change_then_fail),
        }];

        let error = migrate_database(&db, &steps, 11).unwrap_err();

        assert!(error.contains("falha sintética"));
        assert!(error.contains("backup foi restaurado"));
        let conn = open_database(&db).unwrap();
        assert_eq!(read_db_version(&conn).unwrap(), 10);
        let value: i32 = conn
            .query_row(r#"SELECT v FROM "Sentinel""#, [], |r| r.get(0))
            .unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn migration_reports_both_errors_when_restore_is_blocked() {
        let dir = TestDir::new();
        let db = dir.file("teller.db");
        {
            let conn = open_database(&db).unwrap();
            conn.execute_batch(
                r#"CREATE TABLE "App_Meta" ("key" VARCHAR PRIMARY KEY, "value" VARCHAR);
                   INSERT INTO "App_Meta" VALUES ('db_version', '10');"#,
            )
            .unwrap();
        }
        fn block_restore_then_fail(conn: &Connection) -> Result<(), String> {
            let staged = adjacent_path(conn.path().unwrap(), ".restore");
            std::fs::write(staged, b"recuperacao anterior").unwrap();
            Err("falha sintética".to_string())
        }
        let steps = [MigrationStep {
            version: 11,
            kind: MigrationKind::Rust(block_restore_then_fail),
        }];

        let error = migrate_database(&db, &steps, 11).unwrap_err();

        assert!(error.contains("falha sintética"));
        assert!(error.contains("Falha ao preparar a restauração"));
        assert!(error.contains("Backup disponível em"));
        assert_eq!(
            std::fs::read(adjacent_path(&db, ".restore")).unwrap(),
            b"recuperacao anterior"
        );
        let backup = std::fs::read_dir(&dir.0)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("teller.db.bak.10.")
            })
            .unwrap();
        let conn = open_database(&backup).unwrap();
        assert_eq!(read_db_version(&conn).unwrap(), 10);
    }

    // O processo auxiliar termina sem executar os destrutores da conexão.
    #[test]
    #[ignore = "processo auxiliar do teste de recuperação"]
    fn wal_writer_process() {
        let db = std::env::var_os("TELLER_WAL_TEST_PATH").expect("caminho do banco de teste");
        let conn = open_database(Path::new(&db)).unwrap();
        conn.execute_batch(
            "SET wal_autocheckpoint = '1GB';
             CREATE TABLE Sentinel (v INTEGER);
             CHECKPOINT;
             INSERT INTO Sentinel VALUES (42);",
        )
        .unwrap();
        std::process::exit(0);
    }

    #[test]
    fn committed_transaction_survives_abrupt_process_exit() {
        let dir = TestDir::new();
        let db = dir.file("teller.db");
        run_child_test("db::recovery_tests::wal_writer_process", &db);
        let wal = adjacent_path(&db, ".wal");
        assert!(std::fs::metadata(&wal).unwrap().len() > 0);

        let conn = open_database(&db).unwrap();

        let value: i32 = conn
            .query_row("SELECT v FROM Sentinel", [], |r| r.get(0))
            .unwrap();
        assert_eq!(value, 42);
    }
}

#[cfg(test)]
mod migration_tests {
    use super::*;

    fn mini_db(version: u32) -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"CREATE TABLE "App_Meta" ("key" VARCHAR PRIMARY KEY, "value" VARCHAR NOT NULL);"#,
        )
        .unwrap();
        conn.execute(
            r#"INSERT INTO "App_Meta" ("key", "value") VALUES ('db_version', ?)"#,
            duckdb::params![version.to_string()],
        )
        .unwrap();
        conn
    }

    fn noop(_: &Connection) -> Result<(), String> {
        Ok(())
    }
    fn fail(_: &Connection) -> Result<(), String> {
        Err("falha sintética".to_string())
    }

    // Scenario: Banco já na versão alvo
    #[test]
    fn noop_at_target_version() {
        let conn = mini_db(10);
        run_migrations_inner(&conn, &[], 10).unwrap();
        assert_eq!(read_db_version(&conn).unwrap(), 10);
    }

    // Scenario: Banco mais novo que o binário
    #[test]
    fn downgrade_returns_error_and_leaves_db_intact() {
        let conn = mini_db(15);
        let err = run_migrations_inner(&conn, &[], 10).unwrap_err();
        assert!(
            err.contains("Downgrade"),
            "expected downgrade error, got: {err}"
        );
        assert_eq!(read_db_version(&conn).unwrap(), 15);
    }

    // Scenario: Banco sem chave de versão é anômalo
    #[test]
    fn missing_version_is_anomalous() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"CREATE TABLE "App_Meta" ("key" VARCHAR PRIMARY KEY, "value" VARCHAR NOT NULL);"#,
        )
        .unwrap();
        let err = run_migrations_inner(&conn, &[], 7).unwrap_err();
        assert!(
            err.contains("anômalo") || err.contains("ausente"),
            "expected anomalous error, got: {err}"
        );
    }

    // stored < target but no migration steps → error (missing steps)
    #[test]
    fn missing_steps_returns_error() {
        let conn = mini_db(5);
        let err = run_migrations_inner(&conn, &[], 6).unwrap_err();
        assert!(
            err.contains("incompleta") || err.contains("ausentes"),
            "expected missing-steps error, got: {err}"
        );
        // version unchanged
        assert_eq!(read_db_version(&conn).unwrap(), 5);
    }

    // Scenario: Aplicação de um único passo
    #[test]
    fn single_step_stamps_target_version() {
        let conn = mini_db(6);
        let steps = [MigrationStep {
            version: 7,
            kind: MigrationKind::Rust(noop),
        }];
        run_migrations_inner(&conn, &steps, 7).unwrap();
        assert_eq!(read_db_version(&conn).unwrap(), 7);
    }

    // Scenario: Banco dormente salta várias versões
    #[test]
    fn multi_step_jump_applies_all_and_stamps_final() {
        let conn = mini_db(6);
        let steps = [
            MigrationStep {
                version: 7,
                kind: MigrationKind::Rust(noop),
            },
            MigrationStep {
                version: 8,
                kind: MigrationKind::Rust(noop),
            },
            MigrationStep {
                version: 9,
                kind: MigrationKind::Rust(noop),
            },
        ];
        run_migrations_inner(&conn, &steps, 9).unwrap();
        assert_eq!(read_db_version(&conn).unwrap(), 9);
    }

    // Scenario: Falha no meio de uma cadeia de passos
    #[test]
    fn failure_mid_chain_restores_backup_and_returns_error() {
        let tmp = std::env::temp_dir().join(format!(
            "teller_test_{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        // Create DB at v10 with a sentinel row.
        {
            let conn = Connection::open(&tmp).unwrap();
            conn.execute_batch(
                r#"CREATE TABLE "App_Meta" ("key" VARCHAR PRIMARY KEY, "value" VARCHAR NOT NULL);
                   CREATE TABLE "Sentinel" ("v" INTEGER);"#,
            )
            .unwrap();
            conn.execute(
                r#"INSERT INTO "App_Meta" ("key", "value") VALUES ('db_version', '10')"#,
                [],
            )
            .unwrap();
            conn.execute(r#"INSERT INTO "Sentinel" VALUES (42)"#, [])
                .unwrap();
            // conn dropped: checkpoints v10 to file
        }

        // Backup + migrate: step 11 succeeds, step 12 fails.
        {
            let backup = {
                // Drop conn before backup copy to release Windows file lock.
                let stored = {
                    let conn = Connection::open(&tmp).unwrap();
                    let s = read_db_version(&conn).unwrap();
                    s // conn dropped here
                };
                backup_db_file(&tmp, stored).unwrap()
            };

            let steps = [
                MigrationStep {
                    version: 11,
                    kind: MigrationKind::Rust(noop),
                },
                MigrationStep {
                    version: 12,
                    kind: MigrationKind::Rust(fail),
                },
            ];

            let result = {
                let conn = Connection::open(&tmp).unwrap();
                run_migrations_inner(&conn, &steps, 12)
                // conn dropped: checkpoints step-11 commit (db now at v11)
            };

            assert!(result.is_err(), "expected error from failing step");
            restore_backup(&backup, &tmp).unwrap();
            // file is now the v10 backup
        }

        // Verify file restored to v10.
        {
            let conn = Connection::open(&tmp).unwrap();
            let version: String = conn
                .query_row(
                    r#"SELECT "value" FROM "App_Meta" WHERE "key" = 'db_version'"#,
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(version, "10", "version should be 10 after restore");
            let sentinel: i32 = conn
                .query_row(r#"SELECT "v" FROM "Sentinel""#, [], |r| r.get(0))
                .unwrap();
            assert_eq!(sentinel, 42, "sentinel data should be intact");
        }

        // Cleanup temp files.
        let _ = std::fs::remove_file(&tmp);
        if let Some(dir) = tmp.parent() {
            let prefix = format!("{}.bak.", tmp.file_name().unwrap().to_string_lossy());
            if let Ok(entries) = std::fs::read_dir(dir) {
                for e in entries.flatten() {
                    if e.file_name().to_string_lossy().starts_with(&prefix) {
                        let _ = std::fs::remove_file(e.path());
                    }
                }
            }
        }
    }

    // Scenario: Instalação limpa pula migrações
    #[test]
    fn new_install_skips_all_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        let target: u32 = DB_VERSION.parse().unwrap();
        run_migrations_inner(&conn, MIGRATIONS, target).unwrap();
        assert_eq!(read_db_version(&conn).unwrap(), target);
    }

    // Scenario: Banco existente em v7 migra para v8 (is_cash)
    fn v8_test_add_is_cash(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(r#"ALTER TABLE "Payment_Methods" ADD COLUMN "is_cash" BOOLEAN;"#)
            .map_err(|e| e.to_string())?;
        conn.execute(r#"UPDATE "Payment_Methods" SET "is_cash" = false"#, [])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[test]
    fn migration_v7_to_v8_adds_is_cash_column() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"CREATE SEQUENCE payment_methods_id_seq START 1;
               CREATE TABLE "App_Meta" ("key" VARCHAR PRIMARY KEY, "value" VARCHAR NOT NULL);
               CREATE TABLE "Payment_Methods" (
                   "id"         INTEGER   DEFAULT nextval('payment_methods_id_seq') PRIMARY KEY,
                   "name"       VARCHAR   NOT NULL,
                   "active"     BOOLEAN   NOT NULL,
                   "updated_at" TIMESTAMP NOT NULL,
                   "created_at" TIMESTAMP NOT NULL
               );
               INSERT INTO "App_Meta" ("key", "value") VALUES ('db_version', '7');
               INSERT INTO "Payment_Methods" (name, active, updated_at, created_at)
                   VALUES ('Dinheiro', true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);"#,
        )
        .unwrap();

        let steps = [MigrationStep {
            version: 8,
            kind: MigrationKind::Rust(v8_test_add_is_cash),
        }];
        run_migrations_inner(&conn, &steps, 8).unwrap();

        assert_eq!(read_db_version(&conn).unwrap(), 8);
        let is_cash: bool = conn
            .query_row(
                r#"SELECT is_cash FROM "Payment_Methods" WHERE name = 'Dinheiro'"#,
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!is_cash, "existing rows should default to false");
    }
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
        assert_eq!(
            count, 0,
            "App_Meta deve conter apenas db_version e app_version"
        );
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
            params!["P", 1.25_f64, 10.0_f64, 12.5_f64, 0.0732_f64, 0.3456_f64],
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
