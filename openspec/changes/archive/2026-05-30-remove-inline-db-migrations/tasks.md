## 1. Schema e metadados

- [x] 1.1 Adicionar tabela `App_Meta(key TEXT PRIMARY KEY, value TEXT NOT NULL)` ao final de `src/db/schema.sql`
- [x] 1.2 Definir `const DB_VERSION: &str = "1";` em `src-tauri/src/db.rs`
- [x] 1.3 Em `init_schema`, após executar o `SCHEMA`, inserir em `App_Meta` as linhas `db_version = DB_VERSION` e `app_version = env!("CARGO_PKG_VERSION")` (apenas no caminho de banco novo, depois do guard)

## 2. Remoção das migrações

- [x] 2.1 Remover de `src-tauri/src/db.rs` as funções: `ensure_user_settings_table`, `ensure_unit_column`, `ensure_profit_margin_column`, `ensure_loss_mode_column`, `ensure_label_column`, `ensure_sales_products_cost_price`, `ensure_financial_entries_expense_subtype`, `ensure_purchase_items_loss_rate`, `ensure_transaction_fees_installments`, `ensure_sale_payments_fee_snapshot`
- [x] 2.2 Remover a função de backfill `normalize_transaction_fee_installments` e o helper `parse_installments_from_description` (usado só por ela)
- [x] 2.3 Atualizar o `pub use db::{...}` em `src-tauri/src/lib.rs` para exportar apenas `DbState`, `init_schema`, `open_database`
- [x] 2.4 Remover do setup em `src-tauri/src/lib.rs` as 11 chamadas `ensure_*`/`normalize_*`, mantendo `open_database` e `init_schema`. Também removidas as chamadas `ensure_*` redundantes em `auth::check_setup`
- [x] 2.5 Atualizar `db::test_db()` para chamar apenas `init_schema`; remover as chamadas às funções deletadas

## 3. Testes automatizados

- [x] 3.1 Manter/ajustar `legacy_decimal_tests` (cobre as leituras `CAST`, fora do escopo de remoção) e confirmar que ainda compila
- [x] 3.2 Adicionar teste em `db.rs` (`#[cfg(test)]`) verificando que após `init_schema` num banco novo `App_Meta` contém `db_version` e `app_version` com valores não vazios
- [x] 3.3 Rodar `rtk cargo test` e garantir que toda a suíte passa

## 4. Banco de dados (regra de versão do projeto)

- [x] 4.1 Excluir o `teller.db` de desenvolvimento local (versão antiga) para que `init_schema` recrie o banco na nova versão com `App_Meta`
- [x] 4.2 Confirmar boot do app sobre banco recriado: schema completo presente e `App_Meta` populada

## 5. Validação manual (delegar a tester humano)

- [x] 5.1 Tester: iniciar o app com banco novo e verificar que todas as telas (produtos, estoque, vendas, financeiro, configurações) carregam sem erro
- [x] 5.2 Tester: confirmar que nenhuma migração roda no boot (sem erros de `ALTER TABLE` no log) e que operações de escrita funcionam normalmente
