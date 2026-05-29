# Spec: db-open-resilience

## Purpose

Garante que o banco de dados DuckDB seja aberto de forma resiliente, lidando com arquivos WAL pendentes ou corrompidos, e que o WAL seja corretamente checkpointed no encerramento graceful do app.

## Requirements

### Requirement: Abertura do banco com fallback de WAL

O sistema SHALL tentar abrir o banco de dados normalmente e, se a abertura falhar (incluindo por falha no replay do WAL), SHALL deletar o arquivo `.wal` correspondente e tentar abrir novamente antes de retornar erro.

#### Scenario: Abertura normal sem WAL pendente
- **WHEN** o app inicia e não há arquivo `.wal` no disco
- **THEN** o banco abre normalmente sem nenhuma ação adicional

#### Scenario: Abertura com WAL válido
- **WHEN** o app inicia e há um arquivo `.wal` que o DuckDB consegue fazer replay
- **THEN** o banco abre com sucesso após o replay automático do DuckDB

#### Scenario: Abertura com WAL corrompido ou com bug de replay
- **WHEN** o app inicia e há um arquivo `.wal` que falha no replay (ex.: bug do DuckDB 1.4.4)
- **THEN** o sistema deleta o arquivo `.wal`
- **THEN** o sistema reabre o banco sem o `.wal`
- **THEN** o app inicializa normalmente

#### Scenario: Falha mesmo após remoção do WAL
- **WHEN** o banco falha ao abrir mesmo após a remoção do `.wal`
- **THEN** o sistema retorna erro e o Tauri exibe falha de setup (comportamento padrão)

### Requirement: Checkpoint automático no encerramento graceful

O sistema SHALL executar `CHECKPOINT` na conexão DuckDB quando a janela principal for destruída, garantindo que o WAL seja fundido ao arquivo principal antes do processo encerrar.

#### Scenario: Encerramento normal pelo usuário
- **WHEN** o usuário fecha a janela do app normalmente
- **THEN** o sistema executa `CHECKPOINT` na conexão antes de encerrar
- **THEN** nenhum arquivo `.wal` permanece no disco após o processo encerrar

#### Scenario: Encerramento por kill forçado ou crash
- **WHEN** o processo é encerrado sem emitir `WindowEvent::Destroyed` (SIGKILL, crash)
- **THEN** o `CHECKPOINT` NÃO é executado (evento não chega)
- **THEN** o arquivo `.wal` pode permanecer no disco
- **THEN** na próxima abertura, o fallback de WAL lida com a situação
