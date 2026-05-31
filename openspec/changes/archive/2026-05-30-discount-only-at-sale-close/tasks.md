## 1. Backend: close_sale aplica descontos

- [x] 1.1 Em `src-tauri/src/sales.rs`, alterar `do_close_sale_work` para receber `items: &[SaleItemInput]` e, no início (após validar status `open`), para cada item ler `final_price` de `Products`, calcular `discount = clamp(0..1)` e `discounted_price = final_price × (1 − discount)`, e fazer `UPDATE "Sales_Products" SET discount = ?, discounted_price = ? WHERE sale_id = ? AND product_id = ?`
- [x] 1.2 Recalcular `total_value = SUM(discounted_price × quantity)` a partir dos itens e fazer `UPDATE "Sales" SET total_value = ?` antes da validação de pagamento; usar esse `total_value` na checagem `SUM(amount_paid) == total_value`
- [x] 1.3 Alterar a assinatura do command `close_sale` para aceitar `items: Vec<SaleItemInput>` além de `payments`, validar `items` não vazio / `quantity > 0`, e repassar a `do_close_sale_work`
- [x] 1.4 Garantir que o restante do fluxo (Sale_Payments, Production_Orders, estoque, snapshot de cost_price, status='closed') permanece inalterado e dentro da mesma transação

## 2. Backend: testes

- [x] 2.1 Adicionar teste em `#[cfg(test)]` de `sales.rs`: `close_sale` com desconto por item recalcula `total_value` e grava `discount`/`discounted_price` corretos
- [x] 2.2 Adicionar teste: pagamento validado contra o total descontado (pagar o total cheio falha; pagar o total descontado fecha)
- [x] 2.3 Rodar `rtk cargo test` e confirmar que os testes existentes de fechamento continuam passando

## 3. Frontend: tipos e SaleForm (remover desconto)

- [x] 3.1 Em `src/types/sale.ts`, atualizar o payload de fechamento (criar/ajustar tipo para `close_sale` com `items: SaleItemInput[]` e `payments: SalePaymentInput[]`)
- [x] 3.2 Em `src/components/sales/SaleForm.tsx`, remover `globalDiscount`/`globalMode`, `handleGlobalDiscountChange`, o campo de desconto por item, o bloco de desconto global e o indicador de desconto máximo; remover imports `computeMaxDiscount`/`applyGlobalDiscount`
- [x] 3.3 Ajustar `updateItem` para tratar apenas `quantity`; subtotal por item e `orderTotal` passam a usar `final_price × quantity`
- [x] 3.4 No `handleSubmit`, enviar `discount: 0` por item no payload de `create_sale`/`update_sale`

## 4. Frontend: SaleCloseForm (adicionar desconto)

- [x] 4.1 Em `src/components/sales/SalesModule.tsx`, passar `products` e `rates` ao `<SaleCloseForm>`
- [x] 4.2 Em `src/components/sales/SaleCloseForm.tsx`, receber `products: Product[]` e `rates: PricingRates`; carregar itens via `get_sale_items` e montar `CartItem` (cruzando `cost_price`/`expected_loss_rate` de `products`), inicializando `discount` com o valor retornado
- [x] 4.3 Replicar os controles de desconto por item + desconto global (R$/%) + indicador de desconto máximo, reaproveitando `computeMaxDiscount` e `applyGlobalDiscount`
- [x] 4.4 Derivar o total a pagar de `SUM(final_price × qty × (1 − discount))`; usar esse total no resumo do topo e no cálculo de `remaining`/`canSubmit`
- [x] 4.5 No `handleSubmit`, montar `items: SaleItemInput[]` (com descontos) e enviar `{ id, items, payments }` ao `close_sale`

## 5. Verificação

- [x] 5.1 Rodar `rtk bun run test` (Vitest) e `rtk tsc` para garantir tipos e testes de `pricing.ts` ok
- [x] 5.2 Verificar manualmente o fluxo: criar pedido a preço cheio, abrir fechamento, aplicar desconto por item e global, confirmar pagamento contra o total descontado
- [x] 5.3 Verificar que um pedido em aberto pré-existente com desconto carrega o desconto no fechamento e pode ser ajustado