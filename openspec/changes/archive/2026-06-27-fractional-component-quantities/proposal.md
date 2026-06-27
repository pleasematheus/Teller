## Why

A ficha técnica (`Product_Components.quantity`) só aceita inteiros, mas insumos medidos em mL, g, L, etc precisam de quantidades fracionárias. Issue #4: uma "Cópia colorida" não consegue declarar "2.5 mL de tinta ciano" — só papel em unidades inteiras. O flag `Units_Of_Measure.integer` e os helpers `isUnitInteger`/`validateQuantity`/`normalizeQuantity` já existem e já são usados em `PurchaseForm`/`SaleForm`, mas o editor de componentes nunca foi ligado a eles.

## What Changes

- **BREAKING** (schema): `Product_Components.quantity` muda de `INTEGER` para `DECIMAL(10,4)`. Requer migration v8 → v9.
- Backend (`stock.rs`): `add_product_component`, `update_product_component` e a struct `ProductComponent` passam quantidade de `i64` para `f64`; validação `quantity > 0` preservada.
- Backend (`sales.rs`): leitura de `pc.quantity` no consumo de estoque passa a ler `f64` (o cálculo `comp_qty as f64 * quantity` já era float).
- Frontend (`ProductForm.tsx`): editor de componentes passa a usar `parseFloat` + `validateQuantity`/`normalizeQuantity` contra o flag `integer` da unidade do insumo filho; `step`/`min` do input derivados da unidade. Mesmo padrão já presente em `PurchaseForm`/`SaleForm`.
- Insumo com unidade inteira (ex papel/unidade) continua rejeitando decimais; insumo com unidade fracionária (ex tinta/mL) aceita.

## Capabilities

### New Capabilities

(nenhuma)

### Modified Capabilities

- `product-components`: a quantidade de componente passa a ser decimal e validada contra o flag de unidade do insumo (inteiro vs fracionário), em vez de sempre inteiro.

## Impact

- Schema: `src/db/schema.sql` (Product_Components.quantity), `src-tauri/src/db.rs` (`DB_VERSION` 8→9 + step ALTER novo em `MIGRATIONS`).
- Backend: `src-tauri/src/stock.rs` (commands + struct + helpers de teste `add_product_component_in_test`), `src-tauri/src/sales.rs` (leitura de `pc.quantity`).
- Frontend: `src/components/products/ProductForm.tsx`, `src/types/stock.ts` (`ProductComponent.quantity` já é `number`, sem mudança de tipo TS).
- Banco existente em dev precisa ser recriado ou migrado (DuckDB segura lock exclusivo durante startup).
