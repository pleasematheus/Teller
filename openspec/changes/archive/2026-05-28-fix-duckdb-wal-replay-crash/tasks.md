## 1. Fallback de abertura do banco (db.rs)

- [x] 1.1 Criar função `pub fn open_database(path: &std::path::Path) -> Result<Connection, duckdb::Error>` em `db.rs`
- [x] 1.2 Implementar a primeira tentativa: `Connection::open(path)`
- [x] 1.3 Em caso de erro, construir o caminho do WAL com `format!("{}.wal", path.display())` e deletar o arquivo com `std::fs::remove_file` (ignorando erro se não existir)
- [x] 1.4 Implementar a segunda tentativa: `Connection::open(path)` e retornar seu resultado diretamente

## 2. Substituição no setup do Tauri (lib.rs)

- [x] 2.1 Substituir `Connection::open(db_path)?` por `open_database(&db_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?`
- [x] 2.2 Adicionar `use db::open_database` ao bloco `pub use` em `lib.rs`

## 3. Checkpoint automático na saída (lib.rs)

- [x] 3.1 Após `app.manage(DbState(...))`, clonar o app handle: `let app_handle = app.handle().clone()`
- [x] 3.2 Registrar listener com `app.on_window_event(move |_window, event| { ... })`
- [x] 3.3 Dentro do listener, verificar `if let tauri::WindowEvent::Destroyed = event`
- [x] 3.4 Acessar o estado via `app_handle.state::<DbState>()`
- [x] 3.5 Travar o mutex e executar `let _ = conn.execute_batch("CHECKPOINT")`

## 4. Verificação

- [x] 4.1 Rodar `cargo build` sem erros de compilação
- [x] 4.2 Abrir o app (`cargo tauri dev`), fechar e reabrir — confirmar que abre normalmente
- [x] 4.3 Confirmar que o arquivo `.wal` não existe após fechar o app normalmente
- [x] 4.4 Simular WAL corrompido criando um `.wal` inválido manualmente e confirmar que o app abre com o fallback
