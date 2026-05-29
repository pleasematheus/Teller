## Context

Hoje o app tem `Payment_Methods` 1:N `Transaction_Fees` (`fee_percent`, `fixed_fee`, `description` livre) e grava `Sale_Payments.net_amount`. Mas:

- `do_close_sale_work` (`sales.rs`) aplica `MAX(fee_percent)` do método a toda venda e ignora `fixed_fee`.
- O checkout (`SaleCloseForm`) só escolhe método + valor — não há modalidade/parcela.
- A taxa não entra no preço: `get_pricing_rates.cv` (`products.rs`) e `computePrice` (`pricing.ts`) não a incluem, mas a DRE (`reports.rs`) desconta `amount_paid − net_amount` → margem some sem aviso.
- `computeMaxDiscount` já reserva `max_fee_percent` no piso — sinal de que a intenção era "taxa embutida", mas o preço cheio nunca a embutiu.

O modelo de negócio escolhido é **repasse difuso via preço**: todos pagam o mesmo preço de tabela, que já carrega a maquininha. A taxa é tratada como **custo variável** e financiada pela média ponderada realizada.

## Goals / Non-Goals

**Goals:**
- Taxas de cartão configuráveis por número de parcelas (tabela por parcela).
- Escolha de modalidade/parcela no fechamento, com taxa visível.
- `net_amount` correto: `amount_paid × (1 − fee_percent) − fixed_fee` da parcela escolhida.
- Taxa embutida no preço somando as taxas pagas ao `cv` (denominador `taxas ÷ faturamento`, 12 meses).
- Unificar piso de desconto e precificação no mesmo `cv`; aposentar `max_fee_percent`.

**Non-Goals:**
- Acréscimo/gross-up no caixa (o cliente não vê "taxa"; preço é preço).
- Antecipação de recebíveis, bandeiras, ou taxa por bandeira.
- Repreçificação automática de produtos já cadastrados (preço é recalculado só ao editar/criar).

## Decisions

### 1. Taxa como custo variável dentro do `cv` (sem termo novo)
`cv = (despesas variáveis + Σ taxas de cartão pagas) ÷ faturamento`, sobre os últimos 12 meses. Isso entrega, por construção, o número que **financia exatamente** a maquininha no repasse difuso — já ponderado por volume e diluído pelas vendas em dinheiro/Pix.
- **Alternativa rejeitada**: termo `card_fee` separado embutido na taxa *cheia* de cartão (`taxas ÷ volume_cartão`). Superarrecada quando nem toda venda é cartão e exige novo campo na fórmula. Cair no `cv` reaproveita a janela de 12 meses e a estrutura existente.

### 2. `installments` como chave da taxa; débito e crédito são métodos separados
`Transaction_Fees` ganha `installments INTEGER NOT NULL DEFAULT 1` (1 = débito/à vista; 2..N = parcelado). Unicidade lógica por `(payment_method_id, installments)`.
- **Alternativa rejeitada**: enum `modality` + `installments` no mesmo método. Desnecessário — "Débito" e "Crédito" já são métodos distintos (o modelo "método = a palavra que o lojista fala"), então não há colisão entre débito e crédito à vista.

### 3. Snapshot da taxa aplicada em `Sale_Payments`
Gravar `applied_fee_percent` e `applied_fixed_fee` no `Sale_Payment` (além de `net_amount`).
- **Por quê**: relatórios estáveis mesmo se a taxa for editada/excluída depois; explica o "porquê" do `net_amount`.
- **Alternativa rejeitada**: FK para `Transaction_Fees`. Frágil a edição/exclusão da taxa.

### 4. Aposentar `max_fee_percent`
Remover de `list_payment_methods` e do piso em `computeMaxDiscount` (que passa a usar só `cv + df`, já com a taxa). O `MAX(fee_percent)` em `close_sale` deixa de existir (substituído pela taxa da parcela).
- **Por quê**: evita reservar a taxa duas vezes; o `cv` unificado é a única fonte.

### 5. Fallback de referência sem histórico
Sem pagamentos em cartão no período, a parcela de cartão do `cv` usa uma taxa de referência derivada da tabela (à vista ou a maior, a confirmar), até haver realizado — espelha o padrão `no_sales_history` já usado por `get_realized_margin`.

## Risks / Trade-offs

- **Margem oscila mês a mês** (a média é histórica) → janela de 12 meses suaviza; o preço só recompõe ao precificar, não silenciosamente.
- **Cold start impreciso** (numerador = 0 sem cartão) → fallback de referência conservador até juntar dados.
- **Preços antigos não recompõem sozinhos** (`final_price` é gravado) → documentar; botão opcional "repreçificar" fica fora de escopo.
- **Migração de taxas legadas** (descrição livre → `installments`) → script best-effort com `installments = 1` por padrão e revisão manual.
- **Recursão cv↔preço** (embutir taxa sobe preço → sobe faturamento → muda cv) → usar realizado histórico (não recursivo); converge na prática.
- **Split payment com parcelas** → `net_amount` por linha; UI mostra parcela só para métodos de cartão.

## Migration Plan

1. **Schema** (`schema.sql`, idempotente + migração para DBs existentes via abertura resiliente): `ALTER TABLE Transaction_Fees ADD COLUMN installments INTEGER NOT NULL DEFAULT 1`; `ALTER TABLE Sale_Payments ADD COLUMN applied_fee_percent ... DEFAULT 0`, `applied_fixed_fee ... DEFAULT 0`.
2. **Dados**: normalizar `Transaction_Fees.description` → `installments` (parse de "Nx" quando possível; senão 1).
3. **Backend antes do frontend** quebra o contrato de `list_payment_methods` (sai `max_fee_percent`, entra a tabela) e de `SalePaymentInput` (entra `installments`) — atualizar tipos TS junto.
4. **Rollback**: colunas novas têm default e são ignoráveis; reverter código restaura o comportamento anterior (com o bug do MAX).

## Open Questions

- Referência de fallback: crédito à vista (1x) ou a maior taxa (conservador)?
- Número máximo de parcelas na UI/validação (12? 18?).
- `description` em `Transaction_Fees`: remover ou manter como rótulo opcional derivado de `installments`?
- Pix/dinheiro: confirmados como métodos sem `Transaction_Fees` (taxa 0), sem necessidade de linha.
