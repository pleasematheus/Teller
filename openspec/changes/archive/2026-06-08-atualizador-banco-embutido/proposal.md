## Why

O app precisa evoluir o schema de bancos existentes sem quebrar quando os usuários atualizarem o binário (via futura pipeline de CI/CD e auto-update do Tauri). A direção anterior previa uma ferramenta de atualização externa dedicada; ela duplica o conhecimento do schema (segunda fonte da verdade), exige seu próprio canal de distribuição e re-empacota o DuckDB. Como o app ainda não foi distribuído (não existem bancos de produção), podemos adotar agora a abordagem mais simples e correta: o próprio binário novo migra o banco na primeira inicialização.

## What Changes

- **BREAKING** A direção de migração muda: deixa de ser "app não migra, ferramenta externa resolve" e passa a ser "o binário novo migra o banco existente na inicialização". A ferramenta de atualização dedicada externa é descartada.
- Novo runner de migração embutido no binário principal. Na inicialização, após `open_database`, compara `App_Meta.db_version` (armazenado) com a constante `DB_VERSION` (alvo):
  - `stored == target`: inicia normalmente.
  - `stored > target`: **recusa o boot** (guarda de downgrade; binário mais antigo que o banco não deve corromper dados).
  - `stored < target`: faz backup de `teller.db` uma vez, roda os passos `stored+1..=target` em ordem, e inicia.
- Backup antes de migrar: copia `teller.db` para `teller.db.bak.<stored>.<timestamp>` uma vez antes de qualquer passo. Em falha de qualquer passo: restaura o backup, aborta o boot e expõe o erro. Isso fornece o isolamento que motivaria um processo separado, sem um segundo binário.
- Cada passo roda numa transação e carimba `App_Meta.db_version` ao concluir.
- **v6 é o baseline.** Não existe caminho de upgrade para bancos `< v6` (nenhum banco de produção existe; bancos de dev são recriados). Os passos de migração começam de v6 em diante.
- Passos de migração armazenados como arquivos SQL numerados (`migrations/NNN.sql`) compilados no binário, com escape hatch em Rust para passos que exijam `EXPORT/IMPORT DATABASE` ou backfill procedural (necessário porque o `ALTER` do DuckDB é fraco: alteração de tipo em coluna referenciada por FK é rejeitada, exigindo table-rebuild).
- A cadeia de migrações é **append-only e imutável** uma vez liberada: um arquivo já distribuído nunca é editado, apenas novos são adicionados. Cada passo é autocontido e não chama código da aplicação que evolui.
- Saltos de versão arbitrários (ex.: v6 para v20) funcionam nativamente: o binário inclui todos os passos até seu `DB_VERSION`; um banco dormente caminha sequencialmente para cima numa única execução.
- Instalações novas continuam rodando `schema.sql` (sempre a versão corrente) e pulam todas as migrações.
- CI testa cada passo de migração contra um fixture na versão `n-1`.

## Capabilities

### New Capabilities
- `atualizacao-banco`: runner de migração embutido, forward-only, gated por versão. Cobre detecção de versão, backup-antes-de-migrar, execução sequencial em transação, carimbo de versão, guarda de downgrade, salto de versões e restauração em falha.

### Modified Capabilities
- `db-schema-initialization`: o requisito "App não migra bancos existentes" é invertido. O app passa a migrar bancos existentes via runner embutido na inicialização. O consumidor de `App_Meta.db_version` passa a ser o runner interno, não uma ferramenta externa.

## Impact

- Código: `src-tauri/src/db.rs` (novo runner + integração com `open_database`/`init_schema`), `src-tauri/src/lib.rs` (chamada do runner no setup antes de disponibilizar a conexão), novo diretório `src-tauri/migrations/` ou `src/db/migrations/` com arquivos SQL numerados.
- Build: arquivos de migração embutidos via `include_str!`/`include_dir` em tempo de compilação.
- CI/CD: futura pipeline deve, a cada mudança de schema, adicionar um arquivo de migração novo e incrementar `DB_VERSION` em lockstep; rodar testes de migração por passo.
- Distribuição: usa o auto-update do Tauri (binário único); nenhum segundo executável é distribuído.
- Specs afetadas: `db-schema-initialization` (modificada), nova `atualizacao-banco`.
- Sem dependências novas obrigatórias (avaliar `include_dir` apenas se `include_str!` por arquivo ficar inviável; decisão em design.md).
