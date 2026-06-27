## 1. Schema e migration

- [x] 1.1 Confirmar se DuckDB 1.4.4 suporta `ALTER TABLE "Product_Components" ALTER COLUMN quantity SET DATA TYPE DECIMAL(10,4)` direto; se não, definir fallback add/copy/drop/rename
- [x] 1.2 Atualizar `src/db/schema.sql`: `Product_Components.quantity` `INTEGER` → `DECIMAL(10,4)`
- [x] 1.3 Incrementar `DB_VERSION` 8 → 9 em `src-tauri/src/db.rs`
- [x] 1.4 Adicionar novo step append-only em `MIGRATIONS` (`009.sql` ou step Rust) com o ALTER da 1.1; não editar steps lançados

## 2. Backend (Rust)

- [x] 2.1 `stock.rs`: struct `ProductComponent.quantity` `i64` → `f64` e `row.get` em `list_product_components`
- [x] 2.2 `stock.rs`: `add_product_component` quantidade `i64` → `f64`, validação `quantity <= 0.0`
- [x] 2.3 `stock.rs`: `update_product_component` quantidade `i64` → `f64`, validação `quantity <= 0.0`
- [x] 2.4 `stock.rs`: helpers de teste `add_product_component_in_test` e `add_product_component_in_test_result` para `f64`
- [x] 2.5 `sales.rs`: leitura de `pc.quantity` (query ~linha 407) para `f64`; confirmar `comp_qty as f64 * quantity` segue correto
- [x] 2.6 Atualizar chamadas de teste em `sales.rs` que passam quantidade inteira (linhas ~863, 999, 1115, 1941)

## 3. Backend — testes

- [x] 3.1 `stock.rs`: ler arquivo antes; teste `add_product_component` aceita `2.5` e persiste decimal
- [x] 3.2 `stock.rs`: teste `update_product_component` aceita `0.5`; rejeita `<= 0.0`
- [x] 3.3 `stock.rs`: teste que consumo de estoque com componente fracionário gera `Stock_Movements.quantity` correto

## 4. Frontend (ProductForm)

- [x] 4.1 `ProductForm.tsx`: importar `isUnitInteger`/`validateQuantity`/`normalizeQuantity` de `@/lib/unitQuantity` (padrão de `PurchaseForm`/`SaleForm`)
- [x] 4.2 `handleAddComponent`/`handleUpdateComponentQty`: `parseInt` → `parseFloat`; chamar `validateQuantity` contra `isUnitInteger(units, insumo.unit)` antes do invoke; exibir erro se falhar
- [x] 4.3 Inputs de quantidade (add e edit): `step`/`min` derivados do flag `integer` da unidade do insumo selecionado/editado
- [x] 4.4 Exibição da quantidade na lista usa `fmtQty` (já existe) para mostrar decimais

## 5. Frontend — testes

- [x] 5.1 `tests/lib/unitQuantity.test.ts`: confirmar cobertura de `validateQuantity` para fração em unidade inteira (ler arquivo antes; adicionar caso se faltar)

## 6. Migração de dados e verificação manual (testadores humanos)

- [x] 6.1 Fechar app dev e recriar/migrar o DB local (DuckDB segura lock exclusivo no startup, obs S18) — HUMANO
- [x] 6.2 Manual: migrar DB v8 existente com componentes inteiros → confirmar valores preservados após v9 — HUMANO
- [x] 6.3 Manual: criar produto "Cópia colorida", adicionar tinta (mL) com `2.5` e papel (unidade) com `1`; confirmar que mL aceita decimal e unidade rejeita — HUMANO
- [x] 6.4 Manual: registrar venda do produto composto e conferir baixa de estoque fracionária da tinta — HUMANO

## 7. Release

- [x] 7.1 Bump `version` em `package.json` e `Cargo.toml` (PATCH);