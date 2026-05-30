## 1. Esquema e migração (DB)

- [x] 1.1 Adicionar coluna `"loss_rate" DECIMAL(5,4)` à tabela `Purchase_Items` em `src/db/schema.sql`
- [x] 1.2 Criar migração idempotente `ensure_purchase_items_loss_rate` em `src-tauri/src/db.rs` (`ADD COLUMN IF NOT EXISTS "loss_rate" DECIMAL(5,4)`)
- [x] 1.3 Registrar a migração no setup de `src-tauri/src/lib.rs` e no `test_db` de `db.rs`

## 2. Backend — perda por insumo (Rust)

- [x] 2.1 Reescrever `get_product_loss_rate` (`products.rs`) para `Σperda / (Σvenda + Σperda)` sobre `Stock_Movements (out)` com `created_at >= NOW() - INTERVAL 12 MONTHS`
- [x] 2.2 Teste: taxa = 0 sem unidades resolvidas; taxa correta com venda+perda; entradas `in` não resolvidas não diluem
- [x] 2.3 Teste: movimentos fora da janela de 12 meses são ignorados

## 3. Backend — fixar perda na compra (Rust)

- [x] 3.1 Estender `PurchaseItemInput` com `loss_rate: Option<f64>` (override) em `stock.rs`
- [x] 3.2 Em `do_purchase_work`: por item, resolver a perda efetiva (override ?? `get_product_loss_rate`), gravar em `Purchase_Items.loss_rate` e `UPDATE Products SET expected_loss_rate = ?`
- [x] 3.3 Teste: compra grava `loss_rate` no item e propaga para `Products.expected_loss_rate`
- [x] 3.4 Teste: override prevalece sobre a sugestão

## 4. Backend — rollup do CMV (Rust)

- [x] 4.1 Função que deriva CMV de um produto a partir de `Product_Components`: `Σ child.cost_price/(1 − child.expected_loss_rate) × quantity`; fallback para custo próprio se não houver componentes
- [x] 4.2 Decidir e implementar o ponto de snapshot do custo derivado (Open Question 2): `close_sale` já captura `cost_price` em `Sales_Products` — alinhar para usar o CMV derivado
- [x] 4.3 Teste: rollup com componentes; produto sem componentes usa custo próprio; componente com perda=1 sinaliza CMV indefinido

## 5. Frontend — cálculo compartilhado (TS)

- [x] 5.1 Adicionar `computeCmvFromComponents(components, ...)` em `src/lib/pricing.ts` espelhando a regra do rollup
- [x] 5.2 Teste Vitest correspondente em `src/lib/pricing.test.ts` (rollup, fallback, perda=1)

## 6. Frontend — compra (React)

- [x] 6.1 No `PurchaseForm.tsx`, adicionar campo de perda por item, pré-populado via `get_product_loss_rate` e editável
- [x] 6.2 Incluir `loss_rate` no payload de `create_purchase`

## 7. Frontend — produto (React)

- [x] 7.1 No `ProductForm.tsx`, remover o botão/modo "Calcular" (auto) da taxa de perda
- [x] 7.2 Para produto com componentes, tornar o custo derivado (campo de custo vira leitura) e exibir o CMV do rollup; manter custo manual para produto sem componentes
- [x] 7.3 Exibir aviso quando vendável não tiver ficha técnica nem custo próprio
- [x] 7.4 Resolver o destino de `loss_mode` (Open Question 4) no tipo `Product` e no form

## 8. Verificação

- [x] 8.1 `cargo test` e `bun run test` (Vitest) verdes
- [x] 8.2 QA manual: comprar insumo → perda fixada; abrir foto → CMV reflete perda dos insumos; recomprar insumo com override → preço da foto atualiza no próximo recongelamento
