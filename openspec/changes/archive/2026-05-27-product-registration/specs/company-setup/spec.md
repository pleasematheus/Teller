## ADDED Requirements

### Requirement: Derivação de taxas de precificação

O sistema SHALL calcular CV% e DF% para a fórmula de markup a partir de `Financial_Entries` e do faturamento dos últimos 12 meses em `Sales`.

#### Scenario: Taxas calculadas com histórico
- **WHEN** o comando `get_pricing_rates` é invocado e existem registros em `Sales` dos últimos 12 meses
- **THEN** o sistema retorna `cv`, `df`, `investment_rate` e `profit_margin` onde `cv = Σ amount (is_fixed=false) / faturamento_ref` e `df = Σ amount (type='expense', is_fixed=true) / faturamento_ref`

#### Scenario: Taxas zeradas sem histórico de vendas
- **WHEN** o comando `get_pricing_rates` é invocado e não existem vendas nos últimos 12 meses
- **THEN** o sistema retorna `cv = 0` e `df = 0` com flag `no_sales_history = true`
