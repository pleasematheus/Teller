## Why

As migrações de schema estão embutidas no setup do app como uma pilha crescente de funções `ensure_*` com `ALTER TABLE`, executadas em toda inicialização. Elas só fazem trabalho real em bancos antigos no disco e acoplam a evolução do schema ao ciclo de vida do app. Uma ferramenta dedicada de atualização do banco será construída depois; o app deve parar de auto-migrar e passar a assumir que o banco já está na versão corrente do schema.

## What Changes

- Remover as funções de migração de `src-tauri/src/db.rs`: `ensure_user_settings_table`, `ensure_unit_column`, `ensure_profit_margin_column`, `ensure_loss_mode_column`, `ensure_label_column`, `ensure_sales_products_cost_price`, `ensure_financial_entries_expense_subtype`, `ensure_purchase_items_loss_rate`, `ensure_transaction_fees_installments`, `ensure_sale_payments_fee_snapshot`, e o backfill `normalize_transaction_fee_installments`.
- Remover as 11 chamadas correspondentes e o `pub use` em `src-tauri/src/lib.rs`, mantendo apenas `open_database` + `init_schema`.
- Ajustar `db::test_db()` para não chamar as funções removidas.
- Adicionar tabela de metadados internos do app (`App_Meta`, chave/valor) ao `schema.sql`, populada por `init_schema` em bancos novos com `db_version` e `app_version`.
- **BREAKING**: o app deixa de atualizar bancos legados. Um `teller.db` anterior à versão corrente do schema passa a ser incompatível até a ferramenta dedicada existir; em desenvolvimento o caminho é recriar o banco.

Fora do escopo (mantido como está): as leituras defensivas `CAST(... AS DOUBLE)` nos SELECTs, o retry de WAL em `open_database`, e a criação inicial do schema via `init_schema`.

## Capabilities

### New Capabilities
- `db-schema-initialization`: criação do schema em banco novo, o contrato de que o app nunca migra bancos existentes, e a tabela de metadados internos (`App_Meta`) com versão do app e do banco.

### Modified Capabilities

## Impact

- Código: `src-tauri/src/db.rs` (remoção de funções + testes), `src-tauri/src/lib.rs` (setup e re-exports), `src/db/schema.sql` (nova tabela `App_Meta`).
- Banco DuckDB: nova tabela `App_Meta`; bancos legados sem as colunas adicionadas por migração tornam-se incompatíveis (sem upgrade automático).
- Sem mudança de dependências.
