## Context

`SaleCloseForm.tsx` exibe, no recebimento, um seletor de parcelas cujas opções incluem a taxa (`2x (3.50%)`) e uma linha "você recebe: R$ X" calculada por `computeNetAmount` (aplica `fee_percent`/`fixed_fee` da parcela). Esses números vêm de `PaymentMethod.fees`, retornado por `list_payment_methods`. A taxa da maquininha já está embutida no preço via `cv`, então expor a taxa e o líquido aqui é redundante.

O backend (`close_sale`) recebe `installments` por pagamento, calcula `net_amount` e grava o snapshot da taxa; o `cv` de pricing e a DRE consomem `amount_paid − net_amount`. Nada disso depende do que a UI mostra.

## Goals / Non-Goals

**Goals:**
- Esconder a porcentagem da taxa nas opções de parcela.
- Remover a exibição do valor líquido no recebimento.
- Preservar o seletor de parcelas e o `installments` enviado ao `close_sale`.

**Non-Goals:**
- Não alterar `close_sale`, `net_amount`, snapshot de taxa, `Transaction_Fees`, schema, pricing (`cv`) ou DRE.
- Não remover o cadastro de taxas em Configurações > Pagamentos.

## Decisions

- **Manter o seletor de parcelas, alterar só o rótulo**: as opções passam de `{fee.installments}x ({pct}%)` para `{fee.installments}x`. O `value` continua sendo `fee.installments`, então o backend recebe a mesma parcela. Alternativa descartada: remover o seletor — perderia a escolha de parcelas que o usuário pediu para manter.
- **Remover `computeNetAmount` e o bloco "você recebe"**: a função e o JSX que exibem o líquido saem do componente. `PaymentMethod.fees` ainda é usado para popular as opções de parcela, então o tipo não muda.

## Risks / Trade-offs

- [Operador não vê mais a taxa retida no momento da venda] → A taxa segue registrada e visível na DRE; a decisão de negócio é que esse número não pertence à tela de recebimento.
