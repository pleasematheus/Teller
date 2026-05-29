## Why

As taxas de cartão variam por modalidade (débito/crédito) e número de parcelas, mas hoje o app não captura essa variação: o fechamento aplica sempre a MAIOR taxa do método (`do_close_sale_work` em `sales.rs`), ignora a `fixed_fee` e não há como escolher a parcela no caixa. Pior, a taxa **não é embutida no preço** (não entra no `cv` de `get_pricing_rates` nem em `computePrice`), mas a DRE a desconta de `net_amount` — gerando um vazamento silencioso de margem. Esta mudança embute a taxa de cartão no preço como custo variável (repasse difuso, todos pagam o mesmo preço) e torna o cálculo por parcela preciso de ponta a ponta.

## What Changes

- **Tabela de taxas por parcela**: `Transaction_Fees` ganha `installments` (1 = débito/à vista; 2..N = crédito parcelado). A configuração deixa de usar descrição livre e passa a uma tabela por número de parcelas. **BREAKING** (o formato dos dados de taxa muda; taxas já cadastradas precisam de normalização).
- **Seleção de parcela no caixa**: o fechamento passa a registrar forma + número de parcelas, com a taxa visível em cada opção.
- **`net_amount` correto**: `net_amount = amount_paid × (1 − fee_percent) − fixed_fee` da parcela escolhida — corrige o bug do `MAX(fee_percent)` e passa a usar `fixed_fee`. A taxa aplicada é gravada em `Sale_Payments` como snapshot.
- **Taxa embutida no preço**: o `cv` de `get_pricing_rates` passa a somar as taxas de cartão pagas (`SUM(amount_paid − net_amount)` dos pagamentos em cartão, últimos 12 meses) ÷ faturamento — média ponderada que financia exatamente a maquininha (denominador `fees ÷ faturamento`).
- **Fórmulas unificadas**: `computePrice` e `computeMaxDiscount` usam o `cv` unificado; o termo separado `max_fee_percent` é aposentado.
- **Fallback sem histórico**: enquanto não houver vendas em cartão no período, a parcela da taxa embutida cai numa referência derivada da tabela configurada (ex.: crédito à vista ou pior caso), espelhando o padrão `no_sales_history` já existente.

## Capabilities

### New Capabilities

(nenhuma — esta mudança ajusta comportamentos de capacidades existentes)

### Modified Capabilities

- `configuracoes-pagamentos`: as taxas de transação passam a ser definidas por número de parcelas (`installments`) em vez de descrição livre, com tabela por parcela na UI e nos commands.
- `sales-management`: o fechamento registra forma + parcela e calcula `net_amount` com a taxa da parcela escolhida (+ `fixed_fee`); a listagem de métodos passa a expor a tabela de taxas (substituindo `max_fee_percent`); o indicador de desconto máximo passa a usar o `cv` unificado (sem termo `max_fee_pct` separado).
- `product-management`: a precificação embute a taxa de cartão realizada (média ponderada via `cv`) no preço, com fallback de referência quando não há histórico de cartão.

## Impact

- **Schema** (`src/db/schema.sql`): `Transaction_Fees.installments INTEGER NOT NULL DEFAULT 1`; snapshot de taxa em `Sale_Payments` (`applied_fee_percent`, `applied_fixed_fee`).
- **Rust**: `settings.rs` (taxas por parcela; `list_payment_methods` sem `max_fee_percent`), `sales.rs` (`SalePaymentInput.installments`, cálculo de `net_amount`), `products.rs` (`get_pricing_rates` somando taxas de cartão dos últimos 12 meses).
- **Frontend**: `PagamentosSection.tsx` (tabela por parcela), `SaleCloseForm.tsx` (seletor de parcela com taxa visível), `pricing.ts` (`computeMaxDiscount` sem `max_fee_pct` separado), tipos em `sale.ts` / `product.ts`.
- **Testes**: `pricing.test.ts` (ajuste do piso de desconto), testes Rust de `sales` / `products` / `settings`. A DRE (`reports.rs`) não muda de fórmula, mas seus valores passam a refletir `net_amount` correto.
- **Dados existentes**: taxas já cadastradas (com descrição livre) precisam de migração/normalização para `installments`.
