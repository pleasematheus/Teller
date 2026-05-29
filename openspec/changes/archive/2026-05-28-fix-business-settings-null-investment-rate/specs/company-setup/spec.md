## MODIFIED Requirements

### Requirement: Cadastro de empresa
O sistema SHALL permitir registrar os dados da empresa com nome, taxa de investimento e margem de lucro. Os campos de taxa são opcionais: quando não informados, são armazenados como NULL (distinguindo-se de 0%).

#### Scenario: Cadastro com dados válidos
- **WHEN** o usuário submete nome não-vazio, taxa de investimento e margem de lucro válidos
- **THEN** o sistema insere um registro em `Business_Settings` e retorna sucesso

#### Scenario: Cadastro com campos numéricos em branco
- **WHEN** o usuário submete o formulário com `investmentRate` e/ou `profitMargin` em branco
- **THEN** o sistema salva o registro com `investment_rate = NULL` e/ou `profit_margin = NULL` e retorna sucesso

#### Scenario: Nome vazio
- **WHEN** o usuário submete nome em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Taxa fora do intervalo
- **WHEN** o usuário submete `investment_rate` ou `profit_margin` com valor fora do intervalo 0–100%
- **THEN** o sistema retorna erro de validação sem criar o registro

## ADDED Requirements

### Requirement: Campos de taxa nullable em Business_Settings
O sistema SHALL armazenar `investment_rate` e `profit_margin` como colunas nullable em `Business_Settings`, e SHALL garantir a migração de DBs existentes removendo a constraint NOT NULL dessas colunas.

#### Scenario: DB novo com campos vazios
- **WHEN** o app é iniciado em um banco novo e o usuário cria a empresa sem preencher as taxas
- **THEN** os campos `investment_rate` e `profit_margin` são armazenados como NULL

#### Scenario: Migração de DB existente
- **WHEN** o app é iniciado em um banco existente que possui `investment_rate NOT NULL`
- **THEN** a constraint NOT NULL é removida sem afetar os dados existentes

### Requirement: Taxas null tratadas como zero na precificação
O sistema SHALL tratar `investment_rate = NULL` e `profit_margin = NULL` como `0.0` ao calcular `PricingRates`, sem erro.

#### Scenario: Cálculo com taxas null
- **WHEN** `get_pricing_rates` é invocado e `investment_rate` ou `profit_margin` é NULL no banco
- **THEN** o sistema retorna `investment_rate = 0.0` e/ou `profit_margin = 0.0` em `PricingRates`
