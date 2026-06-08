## Context

A resolução do caminho do banco vive em `src-tauri/src/lib.rs:24-31`, dentro do closure `.setup()` do Tauri. Hoje lê uma única variável `DB_PATH`: presente vira dev, ausente cai em `app_data_dir/teller.db`. A lógica está embutida no setup e depende de `app.path().app_data_dir()`, o que a torna impossível de testar sem instanciar o app Tauri.

`dotenvy::dotenv().ok()` já roda no início de `run()` (lib.rs:19), carregando o `.env` para `std::env`. `db.rs` apenas recebe um `&Path` já resolvido, sem conhecer configuração.

## Goals / Non-Goals

**Goals:**
- Seletor de ambiente explícito (`APP_ENV`) com caminhos dev e prod declaráveis lado a lado.
- Lógica de resolução pura e testável fora do Tauri.
- Preservar o comportamento de produção atual (sem `.env` instalado, banco em `app_data_dir/teller.db`).

**Non-Goals:**
- Nenhuma mudança em `db.rs`, schema, ou instalador.
- Nenhum parâmetro de configuração novo além dos três de caminho de banco. Parâmetros de usuário continuam em `System_Settings`.
- Nenhuma retrocompatibilidade com `DB_PATH`.

## Decisions

**Decisão 1: Função pura `resolve_db_path` separada do efeito colateral de env.**
Assinatura proposta:
```rust
fn resolve_db_path(
    app_env: Option<&str>,
    dev: Option<&str>,
    prod: Option<&str>,
    data_dir: &Path,
) -> PathBuf
```
O `.setup()` lê `std::env::var(...)` e `app_data_dir()` e passa valores já materializados para a função. Assim os 5 cenários da spec testam sem Tauri nem variáveis de ambiente globais (que são compartilhadas entre testes e causam flakiness).
Alternativa considerada: ler env dentro da função. Rejeitada porque `std::env` é estado global mutável e mutfor testes em paralelo torna-se não determinístico.

**Decisão 2: `APP_ENV` ausente ou desconhecido = production.**
Default seguro. Match em `Some("development")` para o ramo dev; qualquer outra coisa (`Some("production")`, `Some("prod")`, `None`, valor digitado errado) cai em produção. Evita que um `.env` corrompido aponte produção para `./teller-dev.db`.
Alternativa: erro em valor desconhecido. Rejeitada por adicionar caminho de falha em startup sem ganho prático.

**Decisão 3: `DB_PATH_DEV` ausente em ambiente dev usa default `./teller-dev.db`.**
Mantém o dev zero-config atual. String vazia também conta como ausente (trim).

**Decisão 4: `DB_PATH_PROD` vazio cai em `app_data_dir/teller.db`.**
Idêntico ao fallback de produção de hoje. `create_dir_all(data_dir)` permanece no setup antes de juntar o nome do arquivo, só no ramo de fallback.

## Risks / Trade-offs

- [Dev esquece de setar `APP_ENV=development` no `.env`] → cai em produção e abre o banco em `app_data_dir`, não no `./teller-dev.db` esperado. Mitigação: `.env.example` documenta e o `.env` commitado de dev já traz `APP_ENV=development`.
- [Caminho relativo `./teller-dev.db` depende do working directory do processo] → comportamento idêntico ao atual (`DB_PATH=./teller-dev.db`), sem regressão. Não mudamos a semântica de caminho relativo.
- [Clean break quebra qualquer `.env` antigo com `DB_PATH`] → aceito; afeta só a máquina de dev. `DB_PATH` antigo passa a ser ignorado, ambiente vira produção (default seguro), não corrompe nada.

## Migration Plan

1. Reescrever `.env` local e `.env.example`.
2. Substituir lib.rs:24-31 pela leitura de env + chamada a `resolve_db_path`.
3. Sem migração de dados: o arquivo de banco em si não muda de formato nem de local em produção.
4. Rollback: reverter o commit; `.env` antigo com `DB_PATH` volta a funcionar.
