## 1. Baseline

- [x] 1.1 Rodar `cargo test` e confirmar que todos os testes passam antes de começar

## 2. Extrair financials.rs

- [x] 2.1 Criar `src-tauri/src/financials.rs` com `FinancialEntry` e os comandos `list_financial_entries`, `list_recurring_suggestions`, `create_financial_entry`, `update_financial_entry`, `delete_financial_entry`
- [x] 2.2 Mover os testes de financial do `mod tests` de `lib.rs` para `#[cfg(test)] mod tests` em `financials.rs`
- [x] 2.3 Remover o código movido de `lib.rs` e adicionar `pub mod financials;`
- [x] 2.4 Qualificar os handlers de financial em `generate_handler![]` com `financials::`
- [x] 2.5 Rodar `cargo test` — deve passar sem falhas

## 3. Extrair products.rs

- [x] 3.1 Criar `src-tauri/src/products.rs` com `Product`, `PricingRates` e os comandos `get_pricing_rates`, `get_product_loss_rate`, `get_realized_margin`, `list_products`, `create_product`, `update_product`; incluir `validate_product`
- [x] 3.2 Mover os testes de produto para `#[cfg(test)] mod tests` em `products.rs`
- [x] 3.3 Remover o código movido de `lib.rs` e adicionar `pub mod products;`
- [x] 3.4 Qualificar os handlers de products em `generate_handler![]` com `products::`
- [x] 3.5 Rodar `cargo test` — deve passar sem falhas

## 4. Extrair settings.rs

- [x] 4.1 Criar `src-tauri/src/settings.rs` com `BusinessInfo`, `PaymentMethod`, `TransactionFee`, `UserInfo` e os comandos `get_business_info`, `update_business_settings`, `list_users`, `change_user_password`, `toggle_user_active`, `create_payment_method`, `update_payment_method`, `list_transaction_fees`, `upsert_transaction_fee`
- [x] 4.2 Mover os testes de settings/payment/users para `#[cfg(test)] mod tests` em `settings.rs`
- [x] 4.3 Remover o código movido de `lib.rs` e adicionar `pub mod settings;`
- [x] 4.4 Qualificar os handlers de settings em `generate_handler![]` com `settings::`
- [x] 4.5 Rodar `cargo test` — deve passar sem falhas

## 5. Extrair auth.rs

- [x] 5.1 Criar `src-tauri/src/auth.rs` com `User`, `UserRow` e os comandos `check_setup`, `create_user`, `check_business_setup`, `create_business_settings`, `find_user`, `login`, `get_user_setting`, `set_user_setting`; incluir `lookup_user`
- [x] 5.2 Definir `insert_user` como `pub(crate)` dentro de `#[cfg(test)]` em `auth.rs`
- [x] 5.3 Mover os testes de auth/login/user_settings para `#[cfg(test)] mod tests` em `auth.rs`
- [x] 5.4 Remover o código movido de `lib.rs` e adicionar `pub mod auth;`
- [x] 5.5 Qualificar os handlers de auth em `generate_handler![]` com `auth::`
- [x] 5.6 Rodar `cargo test` — deve passar sem falhas

## 6. Extrair db.rs

- [x] 6.1 Criar `src-tauri/src/db.rs` com `DbState` e as funções `init_schema`, `ensure_user_settings_table`, `ensure_unit_column`, `ensure_profit_margin_column`, `ensure_label_column`
- [x] 6.2 Definir `test_db` como `pub(crate)` dentro de `#[cfg(test)]` em `db.rs`
- [x] 6.3 Atualizar os `#[cfg(test)] mod tests` dos demais módulos para importar `crate::db::test_db` onde necessário
- [x] 6.4 Remover o código movido de `lib.rs` e adicionar `pub mod db;`
- [x] 6.5 Rodar `cargo test` — deve passar sem falhas

## 7. Extrair stock.rs

- [x] 7.1 Criar `src-tauri/src/stock.rs` com `StockItem`, `PurchaseItemInput`, `StockMovement`, `ProductComponent`, `Purchase` e os comandos `list_stock`, `create_purchase`, `create_stock_adjustment`, `list_stock_movements`, `list_product_components`, `add_product_component`, `remove_product_component`, `update_product_component`, `list_purchases`; incluir `do_purchase_work`
- [x] 7.2 Adicionar import de `products::Product` em `stock.rs` onde necessário
- [x] 7.3 Mover os testes de stock para `#[cfg(test)] mod tests` em `stock.rs`
- [x] 7.4 Remover o código movido de `lib.rs` e adicionar `pub mod stock;`
- [x] 7.5 Qualificar os handlers de stock em `generate_handler![]` com `stock::`
- [x] 7.6 Rodar `cargo test` — deve passar sem falhas

## 8. Extrair sales.rs

- [x] 8.1 Criar `src-tauri/src/sales.rs` com `SaleRow`, `SaleItemInput`, `SalePaymentInput`, `SaleItemRow` e os comandos `list_sales`, `create_sale`, `update_sale`, `close_sale`, `get_sale_items`, `list_payment_methods`; incluir `do_create_sale_work`, `do_close_sale_work`
- [x] 8.2 Adicionar imports de `settings::PaymentMethod` e `settings::TransactionFee` em `sales.rs` onde necessário
- [x] 8.3 Mover os testes de sales para `#[cfg(test)] mod tests` em `sales.rs`
- [x] 8.4 Remover o código movido de `lib.rs` — deve restar apenas `mod`, `use`, `run()` e o `#[cfg(test)]` vazio
- [x] 8.5 Qualificar os handlers de sales em `generate_handler![]` com `sales::`
- [x] 8.6 Rodar `cargo test` — deve passar sem falhas

## 9. Verificação final

- [x] 9.1 Confirmar que `lib.rs` contém apenas `mod`, `use`, `DbState` (se aplicável), `run()` e `generate_handler![]`
- [x] 9.2 Rodar `cargo build` limpo e confirmar que não há warnings não-existentes antes do refactor
- [x] 9.3 Rodar `cargo test` completo uma última vez e confirmar que todos os 74+ testes passam
