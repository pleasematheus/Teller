## Context

O DuckDB cria um arquivo `.wal` (Write-Ahead Log) ao lado do banco de dados principal durante operações de escrita. Em condições normais, o `.wal` é fundido ao arquivo principal (`CHECKPOINT`) quando a conexão é fechada. Se o processo for encerrado antes do `Connection::drop()` executar (Ctrl+C, crash, kill forçado), o `.wal` permanece no disco.

Na próxima abertura, o DuckDB detecta o `.wal` pendente e tenta fazer replay. O DuckDB 1.4.4 tem um bug nesse caminho: lança `DatabaseManager::GetDefaultDatabase with no default database set`, causando panic antes do setup do Tauri completar.

Arquitetura atual relevante:
- `db.rs`: funções de inicialização do schema, sem gerenciamento do ciclo de vida da conexão
- `lib.rs`: abre a conexão com `Connection::open()` diretamente no setup hook do Tauri, gerencia como `DbState(Mutex<Connection>)`

## Goals / Non-Goals

**Goals:**
- App abre na segunda execução mesmo com `.wal` pendente
- Encerramento graceful (janela fechada normalmente) não deixa `.wal` pendente
- Sem mudança nas APIs Tauri expostas ao frontend

**Non-Goals:**
- Corrigir o bug no DuckDB (upstream)
- Recuperar dados perdidos de transações no WAL corrompido
- Backup automático antes do fallback

## Decisions

### Decisão 1: Função `open_database()` com fallback de WAL

**Escolhido**: Extrair abertura do banco para `db::open_database(path) -> Result<Connection, duckdb::Error>` que tenta abrir normalmente e, em caso de erro, deleta o `.wal` e tenta de novo.

**Alternativa descartada**: Deletar o `.wal` preventivamente sempre antes de abrir.
- Problema: destrói a capacidade de recuperação de crash em cenários onde o DuckDB conseguiria fazer replay com sucesso (quando não há o bug).

**Caminho do `.wal`**: DuckDB usa `<db_path>.wal` (não substitui extensão). Ex.: `teller-dev.db` → `teller-dev.db.wal`. Construir com `format!("{}.wal", db_path.display())`.

### Decisão 2: CHECKPOINT via `WindowEvent::Destroyed`

**Escolhido**: No `setup` hook, clonar o `AppHandle` e registrar listener com `app.on_window_event`. Quando `WindowEvent::Destroyed` for emitido, acessar `app_handle.state::<DbState>()`, travar o mutex, e executar `conn.execute_batch("CHECKPOINT")`.

**Alternativa descartada**: `WindowEvent::CloseRequested`
- Problema: pode ser cancelado pelo usuário (`api.prevent_close()`). `Destroyed` é irrevogável.

**Alternativa descartada**: Hook de sinal do OS (SIGTERM/Ctrl+C handler)
- Problema: complexidade significativa no Windows, e não cobre todos os cenários de encerramento do Tauri. O `Destroyed` do Tauri é mais confiável para encerramentos normais.

**Nota**: O CHECKPOINT não roda em SIGKILL ou crash — é por isso que o fallback de WAL (Decisão 1) também é necessário.

## Risks / Trade-offs

**[Perda de dados no fallback]** → Aceitável: ocorre apenas após crash/kill forçado onde o DuckDB não consegue replay. Os dados no `.wal` eram transações que não foram confirmadas no arquivo principal — o mesmo que seria perdido em qualquer crash de banco sem WAL.

**[CHECKPOINT bloqueia o mutex]** → Mitigado: CHECKPOINT é rápido (flush de memória para disco). O mutex é travado por no máximo alguns milissegundos durante o encerramento, quando nenhuma outra operação de UI deve estar em curso.

**[`Destroyed` pode não ser emitido]** → Mitigado pelo fallback de WAL. Se o processo for morto sem emitir o evento, o fallback garante a abertura na próxima execução.

## Migration Plan

Sem migração necessária. Os bancos existentes com `.wal` pendente serão recuperados automaticamente pelo fallback na primeira execução após o deploy.

## Open Questions

- Nenhuma questão em aberto.
