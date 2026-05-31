## Why

Hoje o desconto (por item e global) é aplicado na criação/edição do pedido, mas a decisão comercial de conceder desconto acontece na hora de receber o pagamento. Aplicar desconto cedo demais polui o formulário de pedido e permite que o `total_value` do pedido em aberto não reflita o que será efetivamente cobrado no caixa. Mover o desconto para o fechamento alinha a ferramenta ao fluxo real de atendimento.

## What Changes

- **BREAKING** (UX): O formulário de criação/edição de pedido (`SaleForm`) deixa de exibir desconto por item, desconto global e o indicador de desconto máximo. Pedidos passam a ser criados/editados a preço cheio (`discount = 0`, `discounted_price = price`, `total_value = SUM(price × quantity)`).
- A tela de fechamento (`SaleCloseForm`) passa a exibir os itens do pedido com campo de desconto por item, desconto global (R$/%) e o indicador de desconto máximo por item, reaproveitando `computeMaxDiscount` e `applyGlobalDiscount`.
- O `total` a pagar no fechamento passa a ser o total já descontado; os pagamentos são validados contra esse total descontado.
- **BREAKING** (API): O command `close_sale` passa a receber os itens com os descontos aplicados no fechamento, persistindo `discount`/`discounted_price` por item e recalculando `total_value` do pedido dentro da mesma transação, antes de validar o pagamento.

## Capabilities

### New Capabilities
<!-- Nenhuma capability nova; o comportamento de desconto é realocado dentro de sales-management. -->

### Modified Capabilities
- `sales-management`: o desconto (por item, global e indicador de máximo) deixa de ocorrer na criação/edição do pedido e passa a ocorrer no fechamento; `close_sale` passa a aplicar e persistir os descontos e recalcular o total antes de validar o pagamento.

## Impact

- Frontend: `src/components/sales/SaleForm.tsx` (remoção de desconto), `src/components/sales/SaleCloseForm.tsx` (adição de desconto + carregamento de itens/produtos/taxas), `src/components/sales/SalesModule.tsx` (passar `products` e `rates` ao `SaleCloseForm`), `src/types/sale.ts` (payload de `close_sale`).
- Backend: `src-tauri/src/sales.rs` — assinatura e lógica de `close_sale`/`do_close_sale_work` para receber itens descontados, recalcular `total_value` e persistir `discount`/`discounted_price`.
- Sem novas dependências. Sem mudança de schema (colunas `discount`/`discounted_price`/`total_value` já existem).
