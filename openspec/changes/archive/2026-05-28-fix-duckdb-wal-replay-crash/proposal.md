## Why

O app entra em panic na segunda execução porque o DuckDB 1.4.4 tem um bug ao fazer replay do WAL (Write-Ahead Log) quando o processo anterior foi encerrado sem fechar a conexão corretamente. Isso afeta qualquer ambiente de dev (Ctrl+C no `cargo tauri dev`) e pode afetar produção em caso de crash do processo.

## What Changes

- Nova função `open_database(path)` em `db.rs` com fallback de WAL: tenta abrir normalmente; se falhar, deleta o `.wal` e tenta novamente
- Listener de `WindowEvent::Destroyed` no setup do Tauri que executa `CHECKPOINT` antes da conexão ser dropada, evitando que o `.wal` fique pendente em encerramentos normais

## Capabilities

### New Capabilities

- `db-open-resilience`: Abertura do banco de dados com recuperação de WAL corrompido e checkpoint automático na saída

### Modified Capabilities

<!-- Nenhuma capability existente tem requisitos alterados — esta é uma mudança puramente de infraestrutura interna -->

## Impact

- `src-tauri/src/db.rs`: nova função pública `open_database()`
- `src-tauri/src/lib.rs`: usa `open_database()` no setup; registra listener de checkpoint na saída
- Sem impacto em APIs Tauri expostas ao frontend
- Sem impacto em schema de banco de dados
