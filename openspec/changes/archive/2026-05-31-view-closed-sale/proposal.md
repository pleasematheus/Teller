## Why

No histórico de vendas (`SalesModule` aba "Histórico"), uma venda fechada não pode ser aberta: a `SalesList` em `mode="history"` recebe `onView={() => {}}` (`SalesModule.tsx:152`) e renderiza apenas `#id`, `label`, `total` e data, sem botão de ação (`SalesList.tsx:46-50`). Depois de fechar um pedido, o operador não tem como revisar o que foi vendido, quais descontos foram aplicados nem como foi pago.

Os dados existem no banco — `Sales_Products` guarda `discount`/`discounted_price` e `Sale_Payments` guarda `amount_paid`/`net_amount`/snapshot de taxa — mas só `get_sale_items` é consultável; não há command para ler os pagamentos de uma venda.

Esta mudança entrega apenas a **visualização somente leitura** (recibo). Reabertura/devolução de venda fechada fica para uma spec futura.

## What Changes

- **Novo command `get_sale_payments`.** Retorna, para um `sale_id`, cada linha de `Sale_Payments` com o nome do método (JOIN `Payment_Methods`), `amount_paid` e `net_amount`. Somente leitura, não altera estado.
- **Linha do histórico ganha ação "Ver".** Em `SalesList` (`mode="history"`), cada pedido passa a ter um botão "Ver" que dispara `onView(sale)`; a data continua exibida.
- **Novo componente `SaleDetail` (recibo read-only).** Mostra os itens (preço cheio, desconto aplicado, subtotal descontado), o total da venda e os pagamentos (forma de pagamento + valor). Sem nenhum campo editável nem botão de salvar/editar/fechar.
- **`SalesModule` ganha um `FormContent` de detalhe.** Novo tipo `{ kind: "detail-sale"; sale }` que abre o `SaleDetail` na área de form, reusando o padrão de aba já existente (label "Pedido #N", botão × para fechar).

Fora de escopo: reabrir/estornar venda, exibição de parcelas (o `installments` escolhido no fechamento não é persistido em `Sale_Payments` hoje).

## Impact

- Affected specs: sales-management
- Affected code:
  - `src-tauri/src/sales.rs` — novo `get_sale_payments` + `SalePaymentRow`; teste em `#[cfg(test)]`
  - `src-tauri/src/lib.rs` — registrar `get_sale_payments` no handler
  - `src/types/sale.ts` — tipo `SalePaymentRow`
  - `src/components/sales/SaleDetail.tsx` — componente novo
  - `src/components/sales/SalesList.tsx` — botão "Ver" no `mode="history"`
  - `src/components/sales/SalesModule.tsx` — `FormContent` de detalhe + render do `SaleDetail`
- Sem migração de banco: a leitura usa `Sales_Products`, `Sale_Payments` e `Payment_Methods`, que já existem.
