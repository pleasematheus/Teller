## Context

`src-tauri/src/db.rs` mistura três responsabilidades: abertura resiliente (`open_database`), criação inicial do schema (`init_schema`) e onze funções de migração `ensure_*` mais um backfill. O setup em `lib.rs` chama tudo em sequência a cada boot. O `schema.sql` já é "Final" e contém todas as colunas que as migrações adicionam, então em bancos novos as migrações são no-ops; elas só atuam em bancos antigos no disco.

`init_schema` tem um guard: se a tabela `Production_Inputs` existe, retorna cedo sem rodar `schema.sql`. Consequência: o schema só roda uma vez, num banco verdadeiramente vazio. Por isso novas colunas adicionadas ao `schema.sql` nunca chegam a bancos existentes, e é exatamente esse gap que as migrações preenchem hoje.

Uma ferramenta de atualização dedicada será construída depois. Esta mudança remove a auto-migração e introduz um carimbo de versão para essa ferramenta consumir.

## Goals / Non-Goals

**Goals:**
- Remover toda migração estrutural e backfill do fluxo de inicialização do app.
- Manter intactas a abertura resiliente (`open_database`) e a criação inicial (`init_schema`).
- Introduzir `App_Meta` (chave/valor) com `db_version` e `app_version`, populada em bancos novos.
- App passa a assumir que o banco está na versão corrente do schema.

**Non-Goals:**
- Construir a ferramenta de atualização dedicada (trabalho futuro separado).
- Remover ou alterar as leituras defensivas `CAST(... AS DOUBLE)` nos SELECTs.
- Detecção de versão ou recusa de boot em banco incompatível (a ferramenta futura cuida disso).
- Lógica de upgrade que leia `App_Meta` para decidir passos.

## Decisions

### Decisão 1: Remover as funções de migração em vez de desativá-las
Apagar as onze funções `ensure_*` e `normalize_transaction_fee_installments` de `db.rs`, o `pub use` em `lib.rs` e as chamadas no setup. Alternativa considerada: deixar as funções como no-ops vazias. Rejeitada porque mantém ruído e dá falsa impressão de que o app ainda migra. O `schema.sql` é a fonte única da verdade para bancos novos.

### Decisão 2: `App_Meta` como tabela chave/valor
Modelar metadados internos como `App_Meta(key TEXT PRIMARY KEY, value TEXT NOT NULL)`, espelhando o padrão já usado em `User_Settings`. Alternativa: colunas tipadas dedicadas (`db_version INTEGER`, etc.). Rejeitada porque chave/valor acomoda metadados internos futuros sem alterar o schema, e é o padrão que a futura ferramenta pode estender.

### Decisão 3: Origem dos valores de versão
`app_version` vem de `env!("CARGO_PKG_VERSION")` em tempo de compilação. `db_version` é uma constante em código (ex.: `const DB_VERSION: &str = "1";`) que será incrementada manualmente quando o `schema.sql` mudar de forma incompatível. Alternativa: derivar `db_version` de hash do `schema.sql`. Rejeitada por ser opaca para humanos e para a ferramenta de upgrade.

### Decisão 4: População de `App_Meta` só em bancos novos
`init_schema` insere as linhas de versão quando cria o schema. Como `init_schema` já tem o guard que pula bancos existentes, isso naturalmente só popula bancos novos. Bancos legados não recebem `App_Meta` por este fluxo; preenchê-los é tarefa da ferramenta dedicada. Não usar `INSERT OR REPLACE` no boot para evitar reescrever versão em cada inicialização.

## Risks / Trade-offs

- [Bancos legados quebram sem upgrade automático] → Aceito e documentado como BREAKING. Em dev recria-se o banco; em produção a ferramenta dedicada (futura) resolve. Pré-condição: a ferramenta deve existir antes de distribuir a usuários com bancos antigos.
- [`db_version` desatualizada por esquecimento ao editar `schema.sql`] → Mitigar com checklist/regra no CLAUDE.md (alterações de banco já exigem tasks de versão) e bump manual explícito em toda mudança de schema.
- [`init_schema` não popula `App_Meta` em bancos já existentes que rodarão a versão nova] → Esperado; esses bancos ficam sem `App_Meta` até a ferramenta de upgrade rodar. O app não depende de `App_Meta` para funcionar nesta fase.

## Migration Plan

Regra do projeto (CLAUDE.md): alterações no banco DuckDB exigem tasks de exclusão da versão antiga e criação da nova. Como `App_Meta` é tabela nova e o restante é remoção de código:
1. Adicionar `App_Meta` ao `schema.sql` e a inserção de versões em `init_schema`.
2. Em desenvolvimento, apagar o `teller.db` local e deixar `init_schema` recriar o banco na nova versão.
3. Remover funções, re-exports e chamadas de migração.
4. Sem rollback de dados necessário (mudança de código + tabela aditiva); reverter = restaurar o commit anterior e recriar o banco.
