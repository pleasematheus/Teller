## Why

Hoje o caminho do banco é controlado por uma única variável `DB_PATH`: se presente, é dev; se ausente, é produção. Esse acoplamento entre "presença da variável" e "ambiente" é implícito e frágil, e não permite declarar caminhos de dev e prod lado a lado no mesmo arquivo de configuração.

## What Changes

- **BREAKING**: Remover a variável `DB_PATH`. Sem retrocompatibilidade.
- Introduzir seletor explícito `APP_ENV` com valores `development` ou `production`.
- Introduzir `DB_PATH_DEV` e `DB_PATH_PROD` para declarar ambos os caminhos no mesmo `.env`.
- Resolução:
  - `APP_ENV=development` usa `DB_PATH_DEV`, ou default `./teller-dev.db` se ausente.
  - `APP_ENV=production` usa `DB_PATH_PROD` se não vazio, senão `app_data_dir/teller.db`.
  - `APP_ENV` ausente ou desconhecido é tratado como `production` (default seguro: um `.env` faltante ou corrompido nunca aponta prod para um caminho de dev).
- Extrair a lógica para uma função pura `resolve_db_path` testável fora do Tauri.
- Atualizar `.env` e `.env.example` para o novo formato.

## Capabilities

### New Capabilities
- `configuracao-ambiente`: Resolução do caminho do banco de dados a partir de variáveis de ambiente (`APP_ENV`, `DB_PATH_DEV`, `DB_PATH_PROD`), incluindo defaults e fallback para `app_data_dir`.

### Modified Capabilities
<!-- Nenhuma. db-open-resilience e db-schema-initialization permanecem inalteradas. -->

## Impact

- `src-tauri/src/lib.rs` linhas 24-31: substituir o `match std::env::var("DB_PATH")` pela chamada a `resolve_db_path`.
- `.env`: reescrever do formato `DB_PATH=...` para `APP_ENV` + `DB_PATH_DEV` + `DB_PATH_PROD`.
- `.env.example`: mesmo formato, documentado.
- Sem alteração em `src-tauri/src/db.rs`.
- Sem bump de versão de schema do banco.
- Sem alteração no instalador Windows (produção continua sem `.env`, caindo no fallback `app_data_dir/teller.db`).
- Dependência `dotenvy` já presente, sem novas dependências.
