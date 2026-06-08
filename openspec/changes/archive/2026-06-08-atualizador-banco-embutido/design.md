## Context

`src-tauri/src/db.rs` hoje tem `open_database` (abertura resiliente que limpa WAL corrompido), `init_schema` (cria `schema.sql` em banco novo, com guard que pula bancos existentes) e a constante `DB_VERSION` (atualmente `"6"`). `App_Meta` (chave/valor) carimba `db_version` e `app_version` em bancos novos. A mudança `2026-05-30-remove-inline-db-migrations` removeu todas as funções `ensure_*` e documentou que uma "ferramenta de atualização dedicada externa" cuidaria do upgrade. Esta mudança substitui essa direção por um runner embutido no próprio binário.

Fatos que moldam o design:
- O app ainda não foi distribuído: não existem bancos de produção em versões `< 6`. v6 é o baseline.
- O fluxo de update planejado é: o auto-update do Tauri troca o binário primeiro; na próxima inicialização, o binário novo migra o banco.
- DuckDB 1.4.4 (bundled). `ALTER ... SET DATA TYPE` é rejeitado em colunas referenciadas por FK (comprovado em `legacy_decimal_tests` de `db.rs`), exigindo table-rebuild para mudanças de tipo/constraint. DDL transacional do DuckDB é parcial.
- `execute_batch` aborta o DuckDB quando objetos já existem, mesmo com `IF NOT EXISTS` (motivo do guard atual em `init_schema`).

## Goals / Non-Goals

**Goals:**
- Migrar bancos existentes para a versão corrente do schema na inicialização do binário, de forma sequencial e forward-only.
- Isolar falhas via backup-antes-de-migrar e restauração em erro, sem um segundo processo/binário.
- Suportar saltos de versão arbitrários (v6 -> vN) numa única execução.
- Manter `schema.sql` como fonte única para bancos novos; manter `open_database` e `init_schema` intactos no comportamento atual de banco novo.
- Guarda de downgrade: recusar boot se o banco for mais novo que o binário.

**Non-Goals:**
- Caminho de upgrade para bancos `< v6` (não existem; dev recria).
- Auto-update do binário em si (responsabilidade do Tauri updater; fora desta mudança).
- Migração para baixo (downgrade de schema). Não suportada; só guardada.
- Backfill que dependa de regras de negócio vivas do app (proibido por design; ver Decisão 4).

## Decisions

### Decisão 1: Runner embutido no binário principal, não app separado
O binário novo já carrega `DB_VERSION` e `schema.sql`; ele é a única fonte da verdade do alvo. Um app separado duplicaria esse conhecimento, re-empacotaria o DuckDB e exigiria canal de update próprio. Alternativa (app dedicado externo, direção anterior): rejeitada por drift de schema entre dois binários, complexidade de distribuição e por não resolver isolamento melhor do que um backup. O isolamento desejado vem do backup, não do processo separado.

### Decisão 2: Gating por `App_Meta.db_version` vs constante `DB_VERSION`
Na inicialização, ler `stored = App_Meta.db_version` e comparar com `target = DB_VERSION`:
- `stored == target`: prosseguir.
- `stored > target`: recusar boot (downgrade guard) com erro claro; não tocar no banco.
- `stored < target`: backup uma vez, rodar passos `stored+1..=target`, prosseguir.
Ordem no setup: `open_database` -> `init_schema` (no-op em banco existente) -> `run_migrations` -> disponibilizar conexão. Em banco novo, `init_schema` carimba `db_version = DB_VERSION`, então `run_migrations` vê `stored == target` e é no-op. Banco sem a chave `db_version` em `App_Meta`: tratar como erro de boot (banco anômalo; não inferir versão), já que todo banco legítimo `>= v6` a possui.

### Decisão 3: Passos como arquivos SQL numerados + escape hatch em Rust
Passos em `migrations/NNN.sql` (ex.: `007.sql`), numerados pela versão de destino, embutidos via `include_str!` em tempo de compilação (registrados numa lista ordenada no código; sem leitura de disco em runtime). O caso comum (ALTER aditivo, table-rebuild via SQL) é coberto por SQL puro. Para passos que precisem de `EXPORT DATABASE`/`IMPORT DATABASE` ou backfill procedural, um passo pode ser uma função Rust em vez de arquivo SQL. O runner trata uma lista ordenada de passos onde cada item é "SQL embutido" ou "fn Rust"; ele não se importa com qual. Alternativa (só Rust `ensure_*`): rejeitada por ser o spaghetti removido. Alternativa (só SQL): rejeitada porque `EXPORT/IMPORT` precisa de código host. Avaliar `include_dir` apenas se a lista manual de `include_str!` ficar grande; preferir `include_str!` explícito por simplicidade e zero dependência nova.

### Decisão 4: Cadeia append-only, imutável e autocontida
Uma vez liberado a usuários, um arquivo de migração nunca é editado; só se adicionam novos. Editar um passo já aplicado divergiria bancos de usuários dormentes dos já atualizados. Cada passo assume o schema exatamente como estava na versão `n-1` e não chama código da aplicação (precificação, etc.), que evolui: qualquer lógica necessária é congelada dentro do passo. A ordem é load-bearing (um passo pode remover coluna criada por passo anterior); a cadeia nunca é "colapsada/otimizada".

### Decisão 5: Backup único antes da execução, restauração em falha
Antes do primeiro passo, copiar `teller.db` para `teller.db.bak.<stored>.<timestamp>` (uma vez, não por passo). Cada passo roda em transação e, ao concluir, faz `UPDATE App_Meta SET value = <n> WHERE key = 'db_version'` no mesmo commit. Se qualquer passo falhar: restaurar o arquivo de backup sobre `teller.db`, abortar o boot e expor o erro (não deixar o banco "preso" numa versão intermediária com binário que a recusa). Backup uma vez (não por passo) evita lixo e mantém a restauração trivial: voltar a `bak.<stored>`.

### Decisão 6: Bump de `DB_VERSION` em lockstep com novo arquivo
Toda mudança incompatível de `schema.sql` exige (a) novo `migrations/NNN.sql` (ou passo Rust) e (b) incremento de `DB_VERSION` para `N`. Regra do CLAUDE.md (mudanças de banco exigem tasks de exclusão/criação) é estendida para incluir o arquivo de migração. CI valida que existe um passo para cada salto de versão entre o mínimo suportado (6) e `DB_VERSION`.

## Risks / Trade-offs

- [Editar um passo já liberado diverge bancos de usuários] → Cadeia append-only e imutável (Decisão 4); CI bloqueia modificação de arquivos `migrations/NNN.sql` já existentes (checagem de hash em PR, trabalho futuro de CI).
- [DDL transacional parcial do DuckDB deixa banco inconsistente em falha] → O backup-antes-de-migrar é a rede de segurança real, não a transação. Em qualquer falha, restaura `bak.<stored>` e aborta.
- [Salto longo (ex.: 14 passos) bate em edge case de dado real] → Cada passo testado em CI contra fixture na versão `n-1`; o backup permite reportar sem perda de dados.
- [`ALTER` fraco do DuckDB em colunas FK] → Padrão table-rebuild documentado (criar tabela nova, `INSERT ... SELECT`, dropar FKs/old, renomear, recriar FKs); `EXPORT/IMPORT DATABASE` como opção nuclear via passo Rust.
- [Esquecer de bumpar `DB_VERSION` ao editar `schema.sql`] → Regra no CLAUDE.md + bump manual explícito; banco novo carimba a versão errada se esquecido. Mitigar com checagem de CI (Decisão 6).
- [Banco mais novo que binário após rollback do app] → Guarda de downgrade recusa boot com mensagem clara em vez de corromper (Decisão 2).
- [Falha ao copiar backup (disco cheio/permissão)] → Abortar antes de tocar no banco; sem backup, sem migração. Erro visível.

## Migration Plan

Regra do CLAUDE.md: mudanças no banco DuckDB exigem tasks de exclusão da versão antiga e criação da nova.
1. Adicionar o runner (`run_migrations`) e a infraestrutura de passos a `db.rs`; integrar no setup de `lib.rs` após `init_schema`.
2. Criar o diretório de migrações vazio (baseline v6; nenhum passo `<= 6`). O primeiro passo real (`007.sql`) só nasce na próxima mudança incompatível de schema.
3. Em desenvolvimento, apagar `teller.db` local e deixar `init_schema` recriar na versão corrente (v6), carimbando `App_Meta`.
4. Testes unitários do runner usando bancos in-memory carimbados em versões sintéticas e passos de migração de teste.
5. Rollback: como não há passo real ainda, reverter = restaurar o commit anterior. Após existirem passos, o rollback de dados é o backup `bak.<stored>` restaurado pelo próprio runner em falha.

## Open Questions

- Local do diretório de migrações: `src-tauri/migrations/` (junto do crate Rust) vs `src/db/migrations/` (junto de `schema.sql`). `schema.sql` hoje vive em `src/db/`; manter coesão sugere `src/db/migrations/`. Decidir na implementação.
- Política de retenção dos arquivos `teller.db.bak.*`: manter sempre, ou limpar após boot bem-sucedido confirmado? Proposta: manter o último por segurança; não bloquear esta mudança.
- Formato exato do erro de downgrade exposto ao usuário (mensagem na splash/login). Definir na task de UI quando houver superfície.
