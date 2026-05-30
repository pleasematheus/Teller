## Why

A taxa de cartão já está embutida no preço de venda (o `cv` de `get_pricing_rates` financia a maquininha). Mostrar a porcentagem da taxa e o "valor líquido" na tela de recebimento confunde o operador e expõe um número que não muda a decisão de cobrança. O operador só precisa escolher a quantidade de parcelas e ver o total da compra.

## What Changes

- Remover a porcentagem da taxa exibida em cada opção de parcela no seletor do recebimento (ex.: `2x (3.50%)` passa a `2x`).
- Remover a linha "você recebe: R$ X" (valor líquido) do recebimento.
- Manter o seletor de quantidade de parcelas e o total do pedido.
- Nenhuma mudança de backend: `close_sale` continua recebendo `installments`, calculando `net_amount` e gravando o snapshot da taxa; pricing (`cv`) e DRE seguem inalterados.

## Capabilities

### New Capabilities
<!-- Nenhuma capability nova. -->

### Modified Capabilities
- `sales-management`: o requisito de seleção de modalidade/parcela no fechamento deixa de exibir a taxa por parcela e o valor líquido na UI; a parcela escolhida continua determinando a taxa registrada no backend.

## Impact

- Frontend: `src/components/sales/SaleCloseForm.tsx` (rótulo das opções de parcela sem `%`; remoção do bloco "você recebe" e da função `computeNetAmount`).
- Specs: `openspec/specs/sales-management/spec.md` (cenários do requisito "Seleção de modalidade e parcela no fechamento").
- Sem mudança de backend, schema, pricing ou DRE.
