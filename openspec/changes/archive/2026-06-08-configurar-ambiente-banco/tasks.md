## 1. Função pura de resolução

- [x] 1.1 Adicionar `fn resolve_db_path(app_env: Option<&str>, dev: Option<&str>, prod: Option<&str>, data_dir: &Path) -> PathBuf` em `src-tauri/src/lib.rs`
- [x] 1.2 Implementar ramo development: `dev` não vazio (após trim) usa o valor, senão `./teller-dev.db`
- [x] 1.3 Implementar ramo production: `prod` não vazio (após trim) usa o valor, senão `data_dir.join("teller.db")`
- [x] 1.4 Tratar `Some("development")` como dev; qualquer outro valor, `None`, ou desconhecido como production

## 2. Integração no setup do Tauri

- [x] 2.1 Substituir o bloco `match std::env::var("DB_PATH")` em lib.rs:24-31 por leitura de `APP_ENV`, `DB_PATH_DEV`, `DB_PATH_PROD` e chamada a `resolve_db_path`
- [x] 2.2 Manter `create_dir_all` apenas no ramo de fallback que usa `app_data_dir`
- [x] 2.3 Confirmar que `db.rs` permanece inalterado

## 3. Arquivos de ambiente

- [x] 3.1 Reescrever `.env` para `APP_ENV=development`, `DB_PATH_DEV=./teller-dev.db`, `DB_PATH_PROD=`
- [x] 3.2 Reescrever `.env.example` com os três parâmetros e comentários explicando defaults e fallback de produção

## 4. Testes (Rust, no mesmo arquivo, #[cfg(test)])

- [x] 4.1 Teste: dev + `DB_PATH_DEV` custom retorna o caminho custom
- [x] 4.2 Teste: dev + `DB_PATH_DEV` ausente/vazio retorna `./teller-dev.db`
- [x] 4.3 Teste: production + `DB_PATH_PROD` setado retorna esse caminho
- [x] 4.4 Teste: production + `DB_PATH_PROD` vazio retorna `data_dir/teller.db`
- [x] 4.5 Teste: `APP_ENV` ausente cai em production (fallback `data_dir/teller.db`)
- [x] 4.6 Teste: `APP_ENV` desconhecido (ex `staging`) cai em production
- [x] 4.7 Rodar `cargo test` e confirmar suíte verde (delegar a subagente)

## 5. Validação

- [x] 5.1 `openspec validate configurar-ambiente-banco --strict`
- [x] 5.2 Teste manual (humano): rodar `bun run tauri dev` com `APP_ENV=development` e confirmar que abre `./teller-dev.db`
- [x] 5.3 Teste manual (humano): build de produção sem `.env` e confirmar banco em `app_data_dir/teller.db`
