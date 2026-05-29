## ADDED Requirements

### Requirement: Taxa de cartão embutida na precificação

O sistema SHALL embutir a taxa de cartão no preço dos produtos como custo variável, somando as taxas de cartão pagas ao numerador do `cv` em `get_pricing_rates`, de modo que o preço financie a maquininha (repasse difuso, com denominador `taxas ÷ faturamento`).

#### Scenario: Taxa de cartão somada ao cv com histórico
- **WHEN** `get_pricing_rates` é invocado e há pagamentos em cartão nos últimos 12 meses
- **THEN** `cv = (despesas variáveis + SUM(amount_paid − net_amount) dos pagamentos em cartão dos últimos 12 meses) ÷ faturamento dos últimos 12 meses`

#### Scenario: Fallback sem histórico de cartão
- **WHEN** não há pagamentos em cartão nos últimos 12 meses, mas existem taxas de cartão configuradas
- **THEN** a parcela de cartão do `cv` usa uma taxa de referência derivada da tabela configurada (ex.: crédito à vista ou a maior taxa), até que exista histórico realizado

#### Scenario: Sem cartão e sem taxas configuradas
- **WHEN** não há pagamentos em cartão nem taxas de cartão configuradas
- **THEN** a parcela de cartão do `cv` é 0, sem erro

#### Scenario: Preço final cobre a taxa de cartão
- **WHEN** um produto é precificado com a taxa de cartão presente no `cv`
- **THEN** o `final_price` resultante (CMV × markup) cobre, no agregado, as taxas de cartão retidas, de forma que a margem-alvo se mantém para o mix de pagamento histórico

## MODIFIED Requirements

### Requirement: Precificação automática no formulário

O sistema SHALL calcular e exibir em tempo real CMV, Markup% e Preço Final conforme o usuário edita `cost_price`, `expected_loss_rate` e `profit_margin`, usando as taxas de `get_pricing_rates` (cujo `cv` já inclui a taxa de cartão embutida) e a margem individual do produto.

#### Scenario: Cálculo com histórico de vendas disponível
- **WHEN** o formulário é aberto e existem vendas nos últimos 12 meses
- **THEN** CV% é derivado de (despesas variáveis de `Financial_Entries` + taxas de cartão pagas de `Sale_Payments`) ÷ faturamento dos últimos 12 meses; DF% é derivado das despesas fixas de `Financial_Entries` ÷ faturamento; `profit_margin` é pré-populada com `get_realized_margin`

#### Scenario: Cálculo sem histórico de vendas
- **WHEN** o formulário é aberto e não existem vendas nos últimos 12 meses
- **THEN** CV% = 0, DF% = 0 e `profit_margin = 0`; o formulário exibe aviso de que as taxas não puderam ser calculadas

#### Scenario: Atualização em tempo real
- **WHEN** o usuário altera `cost_price`, `expected_loss_rate` ou `profit_margin`
- **THEN** CMV, Markup% e Preço Final são recalculados instantaneamente no cliente sem nova requisição, usando a `profit_margin` do produto em vez da margem global
