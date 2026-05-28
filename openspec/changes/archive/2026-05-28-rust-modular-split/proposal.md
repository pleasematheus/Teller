## Why

`src-tauri/src/lib.rs` tem 3343 linhas com todo o backend Rust em um único arquivo. As fronteiras de domínio existem apenas como comentários — sem enforcement do compilador — o que permite dependências cruzadas acidentais acumularem silenciosamente. O app ainda não está em produção, tornando este o momento ideal para estabelecer uma arquitetura modular antes que o custo de refatoração aumente.

## What Changes

- `lib.rs` reduzido a declarações de módulos (`mod`), `run()` e `generate_handler![]`
- Novo arquivo `db.rs`: `DbState`, `init_schema`, funções `ensure_*` de migração
- Novo arquivo `auth.rs`: structs `User`/`UserRow`, comandos de autenticação e configurações de usuário
- Novo arquivo `products.rs`: struct `Product`/`PricingRates`, CRUD de produtos e lógica de precificação
- Novo arquivo `stock.rs`: structs de estoque/compra/componentes, CRUD de estoque e movimentações
- Novo arquivo `sales.rs`: structs de venda, comandos de criação/atualização/fechamento de vendas
- Novo arquivo `settings.rs`: `BusinessInfo`, `PaymentMethod`, `TransactionFee`, `UserInfo`, comandos de configuração
- Novo arquivo `financials.rs`: `FinancialEntry`, CRUD de lançamentos financeiros
- Cada módulo recebe seu próprio `#[cfg(test)] mod tests` com os testes pertinentes
- `test_db()` helper movido para `db.rs` como `pub(crate)` para reuso entre módulos de teste
- `insert_user()` helper movido para `auth.rs` como `pub(crate)`
- `generate_handler![]` passa a usar nomes qualificados: `auth::login`, `products::list_products`, etc.

## Capabilities

### New Capabilities

Nenhuma — este é um refactor interno sem mudança de comportamento.

### Modified Capabilities

Nenhuma — nenhuma especificação de comportamento muda. Todas as capabilities existentes continuam funcionando de forma idêntica.

## Impact

- **Código afetado**: `src-tauri/src/lib.rs` inteiro — dividido em 7 novos arquivos
- **API externa**: nenhuma mudança — os comandos Tauri expostos ao frontend permanecem idênticos
- **Testes**: os 74 testes existentes são redistribuídos entre os módulos, sem alteração de lógica
- **Compilação**: após o split, mudanças em um módulo recompilam apenas aquele arquivo (melhoria incremental)
- **Dependências**: nenhuma nova dependência em `Cargo.toml`
