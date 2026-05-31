## Context

O desconto de venda hoje vive inteiramente no `SaleForm`:
- Campo de desconto por item (`updateItem(idx, "discount")`), com indicador de desconto máximo via `computeMaxDiscount`.
- Desconto global em R$/% via `handleGlobalDiscount` + `applyGlobalDiscount`, que distribui o desconto entre os itens.
- O `total_value` enviado a `create_sale`/`update_sale` já vem descontado; o backend persiste `discount` e `discounted_price` por item em `Sales_Products` e `total_value` em `Sales`.

O `SaleCloseForm` apenas coleta pagamentos e valida que `SUM(amount_paid) == sale.total_value`. Ele recebe somente `sale` (sem `products` nem `rates`).

O backend `do_close_sale_work` lê `total_value` já gravado, valida o pagamento, registra `Sale_Payments`, dispara `Production_Orders`, consome estoque e faz snapshot do `cost_price` por item.

Restrições do projeto: lógica de negócio pura em `src/lib/` com testes (`pricing.ts` já tem `computeMaxDiscount`/`applyGlobalDiscount` testados), commands Tauri com teste em `#[cfg(test)]`, sem novas dependências, edições cirúrgicas.

## Goals / Non-Goals

**Goals:**
- Remover toda a UI de desconto do `SaleForm`; pedidos criados/editados a preço cheio.
- Reproduzir no `SaleCloseForm` a capacidade atual: desconto por item, desconto global (R$/%) e indicador de desconto máximo por item.
- O total a pagar e a validação do pagamento usam o total já descontado.
- Persistir os descontos do fechamento em `Sales_Products` (`discount`, `discounted_price`) e recalcular `Sales.total_value`, de forma atômica com o fechamento.

**Non-Goals:**
- Mudar schema do banco (colunas já existem).
- Mudar o cálculo de taxas/`net_amount`, snapshot de `cost_price`, ou consumo de estoque.
- Bloquear descontos abaixo do piso de margem (mantém-se apenas aviso, como hoje).
- Adicionar histórico/auditoria de quem concedeu o desconto.

## Decisions

### Decisão 1: `close_sale` recebe os itens descontados e aplica tudo em uma transação

O `close_sale` passa a receber, além de `payments`, a lista `items: Vec<SaleItemInput>` (mesma struct usada em `create_sale`/`update_sale`: `product_id`, `quantity`, `discount`). Dentro de `do_close_sale_work`, antes da validação do pagamento:
1. Para cada item, lê `final_price` de `Products`, calcula `discounted_price = final_price × (1 − discount)`, atualiza a linha correspondente em `Sales_Products` (`discount`, `discounted_price`) e acumula `total_value`.
2. Atualiza `Sales.total_value`.
3. Segue o fluxo existente (validação de pagamento contra o novo `total_value`, `Sale_Payments`, produção, estoque, snapshot de custo, `status = 'closed'`).

**Por que:** mantém atomicidade (desconto + pagamento + produção em uma transação só) e uma única fonte de verdade para o total cobrado. A struct `SaleItemInput` já existe e é serializável.

**Alternativa considerada — chamar `update_sale` antes de `close_sale` pelo frontend:** reaproveitaria a lógica de reescrita de itens, mas quebra a atomicidade (duas transações separadas) e permite estado inconsistente se o `close_sale` falhar após o `update_sale`. Rejeitada.

**Alternativa considerada — recalcular total só no backend a partir de descontos enviados sem regravar itens:** perderia a consistência de `discounted_price` por item, que os relatórios/DRE consomem. Rejeitada.

### Decisão 2: Reescrever as linhas existentes em vez de DELETE+INSERT

`update_sale` faz `DELETE` + `INSERT` dos itens. No fechamento, os itens não mudam de composição (quantidade/produto vêm do pedido), apenas o desconto. Portanto o `close_sale` faz `UPDATE "Sales_Products" SET discount = ?, discounted_price = ? WHERE sale_id = ? AND product_id = ?`, preservando as linhas (e seus `id`, usados depois para o snapshot de `cost_price`).

**Por que:** evita recriar linhas no meio do fluxo de fechamento que depois lê `Sales_Products.id` para o snapshot de custo. Mais simples e seguro.

### Decisão 3: Frontend — `SaleCloseForm` assume a UI de desconto; `SaleForm` fica a preço cheio

- `SaleForm`: remove `globalDiscount`/`globalMode`, o campo de desconto por item, o indicador de desconto máximo e os imports `computeMaxDiscount`/`applyGlobalDiscount`. O subtotal e o total passam a usar `final_price × quantity`. O payload de `create_sale`/`update_sale` envia `discount: 0` por item (assinatura do backend inalterada).
- `SaleCloseForm`: passa a receber `products: Product[]` e `rates: PricingRates` (de `SalesModule`, que já os carrega). Carrega os itens via `get_sale_items`, monta `CartItem` com `cost_price`/`expected_loss_rate` cruzando com `products`, e replica os controles de desconto por item + global + indicador de máximo. O total a pagar passa a ser `SUM(final_price × qty × (1 − discount))`. No `handleSubmit`, envia `items` (com descontos) e `payments` ao `close_sale`.
- O `total` exibido no topo do `SaleCloseForm` deixa de vir só de `sale.total_value` e passa a ser derivado dos itens com desconto; `remaining` usa esse total.

**Por que:** reaproveita `computeMaxDiscount`/`applyGlobalDiscount` já testados; `SalesModule` já tem `products` e `rates` em estado, então a propagação é trivial.

### Decisão 4: `create_sale`/`update_sale` mantêm a assinatura

Continuam aceitando `discount` em `SaleItemInput` (com `clamp` 0..1). O frontend simplesmente envia `0`. Isso minimiza a superfície de mudança no backend e mantém os testes existentes válidos. A criação/edição produz pedidos a preço cheio porque o frontend não coleta mais desconto.

## Risks / Trade-offs

- [Pedidos antigos em aberto criados com desconto] → Ao abrir o fechamento, o `SaleCloseForm` carrega `discount` real de `get_sale_items`, então descontos pré-existentes aparecem e podem ser ajustados; nada se perde.
- [`close_sale` agora regrava itens — risco de divergência com produção/estoque] → A composição (produto/quantidade) não muda no fechamento, só `discount`/`discounted_price`; o consumo de estoque usa `quantity`, não afetado.
- [Frontend pode enviar itens divergentes do pedido] → O `close_sale` usa `product_id`/`quantity` enviados para recomputar e validar; como o `SaleCloseForm` carrega os itens do próprio pedido, a divergência só ocorreria por bug de UI. Mitigação: o `UPDATE` é por `(sale_id, product_id)`, e itens inexistentes não afetam linhas.
- [Assinatura de `close_sale` muda (BREAKING)] → Único chamador é o `SaleCloseForm`; atualizado no mesmo change.

## Migration Plan

Mudança atômica de código (sem migração de dados). Deploy do frontend e backend juntos. Rollback = reverter o commit; dados gravados permanecem válidos (mesmas colunas).
