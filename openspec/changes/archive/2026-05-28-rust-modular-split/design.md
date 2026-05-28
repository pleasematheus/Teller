## Context

`src-tauri/src/lib.rs` contém 3343 linhas com todo o backend Rust: structs, comandos Tauri, helpers de DB, migrações e 74 testes em um único `mod tests`. Seções lógicas já existem demarcadas por comentários `// ─── Domínio ───`, mas sem enforcement do compilador. O app está em desenvolvimento ativo com planos de novas features e módulos futuros.

## Goals / Non-Goals

**Goals:**
- Dividir `lib.rs` em arquivos por domínio sem alterar nenhuma lógica de negócio
- Estabelecer fronteiras de módulo que o compilador Rust enforce
- Redistribuir os 74 testes para seus módulos correspondentes
- Manter compilação e todos os testes passando a cada passo

**Non-Goals:**
- Alterar qualquer comportamento dos comandos Tauri
- Adicionar novas features durante o refactor
- Mover testes para um diretório `tests/` separado (integration tests)
- Modificar `Cargo.toml` (nenhuma nova dependência)

## Decisions

### D1: Ordem de extração — dos mais isolados para os mais acoplados

Extrair módulos começando pelos que têm menos dependências internas:

```
1. financials.rs  — sem dependências de outros domínios
2. products.rs    — sem dependências de outros domínios  
3. settings.rs    — sem dependências de outros domínios
4. auth.rs        — sem dependências de outros domínios
5. db.rs          — depende de auth (test helper insert_user)
6. stock.rs       — usa tipos de products (Product)
7. sales.rs       — usa tipos de stock e settings (PaymentMethod)
```

**Alternativa considerada**: Extrair tudo de uma vez.
**Razão da rejeição**: Dificulta identificar a origem de erros de compilação; extrair um por vez permite validar `cargo test` a cada passo.

### D2: Visibilidade dos módulos — `pub mod` com re-exports seletivos

Todos os módulos declarados como `pub mod` em `lib.rs`. Tipos usados pelo `generate_handler![]` precisam de visibilidade adequada.

```rust
// lib.rs
pub mod db;
pub mod auth;
pub mod products;
pub mod stock;
pub mod sales;
pub mod settings;
pub mod financials;
```

**Alternativa considerada**: `mod` privado com `pub use` de tudo.
**Razão da rejeição**: `pub mod` é suficiente — o Tauri invoca por caminho qualificado, não precisa de re-export no nível raiz.

### D3: `generate_handler![]` usa nomes qualificados

```rust
tauri::generate_handler![
    auth::check_setup,
    auth::create_user,
    auth::login,
    // ...
    products::list_products,
    products::create_product,
    // ...
]
```

**Alternativa considerada**: Manter nomes sem qualificação via `use module::*`.
**Razão da rejeição**: Nomes qualificados tornam a lista de handlers auto-documentada por domínio.

### D4: Helpers de teste como `pub(crate)`

`test_db()` e `insert_user()` são necessários em múltiplos módulos de teste. Solução: defini-los como `pub(crate)` dentro de um bloco `#[cfg(test)]` no módulo de origem.

```rust
// db.rs
#[cfg(test)]
pub(crate) fn test_db() -> Connection { ... }

// auth.rs
#[cfg(test)]
pub(crate) fn insert_user(conn: &Connection, ...) { ... }
```

**Alternativa considerada**: Duplicar os helpers em cada módulo.
**Razão da rejeição**: Duplicação cria drift — mudanças no schema exigiriam atualizar N cópias.

### D5: Localização de tipos compartilhados

`PaymentMethod`, `TransactionFee` e `UserInfo` estão atualmente na seção Sales mas são **gerenciados** por Settings. Vão para `settings.rs`.

`SalePaymentInput` referencia `payment_method_id` mas pertence ao fluxo de venda — fica em `sales.rs` e importa `PaymentMethod` de `settings`.

## Risks / Trade-offs

**[Risco] Erro de compilação por visibilidade** → Ao mover funções para módulos, funções que eram `fn` privadas e chamadas internamente precisam virar `pub(crate)` se chamadas de outro módulo. Mitigation: compilar após cada extração para identificar imediatamente.

**[Risco] Testes que testam múltiplos domínios** → Alguns testes podem criar dados de um domínio para testar outro (ex: criar produto para testar venda). Mitigation: o teste vai para o módulo que ele está **testando primariamente**; imports cruzados nos testes são permitidos.

**[Trade-off] `pub(crate)` expõe mais** → Funções helper de teste precisam de `pub(crate)`. Isso é aceitável porque: (1) só existe em `#[cfg(test)]`, (2) `pub(crate)` não expõe para o frontend, apenas internamente.
