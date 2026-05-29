## Purpose

Verificação e cadastro dos dados da empresa após o primeiro login, antes de acessar o Dashboard. Cobre a detecção de empresa não cadastrada e o registro de nome, taxa de investimento e margem de lucro.

## Requirements

### Requirement: Verificação de cadastro de empresa
O sistema SHALL verificar se existe ao menos um registro em `Business_Settings` para determinar se o cadastro de empresa já foi realizado.

#### Scenario: Empresa não cadastrada
- **WHEN** a tabela `Business_Settings` não contém registros
- **THEN** o sistema retorna `false` indicando que o cadastro de empresa é necessário

#### Scenario: Empresa já cadastrada
- **WHEN** a tabela `Business_Settings` contém ao menos um registro
- **THEN** o sistema retorna `true` indicando que o cadastro já foi realizado

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
- **WHEN** o usuário submete `investment_rate` ou `profit_margin` fora do intervalo 0–100%
- **THEN** o sistema retorna erro de validação sem criar o registro

### Requirement: Inicialização da tabela User_Settings
O sistema SHALL garantir que a tabela `User_Settings (user_id INTEGER NOT NULL REFERENCES "Users"(id), key TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY (user_id, key))` exista no banco de dados a cada inicialização do app, criando-a se não existir, independentemente do estado do schema principal.

#### Scenario: Tabela criada em DB novo
- **WHEN** o app é iniciado em um banco de dados recém-criado
- **THEN** a tabela `User_Settings` é criada junto com o schema principal

#### Scenario: Tabela garantida em DB existente
- **WHEN** o app é iniciado em um banco de dados já existente que não possui `User_Settings`
- **THEN** a tabela `User_Settings` é criada sem afetar os dados existentes

#### Scenario: Operação idempotente
- **WHEN** o app é iniciado múltiplas vezes em um banco que já possui `User_Settings`
- **THEN** a inicialização completa sem erro e sem duplicar a tabela

### Requirement: Campos de taxa nullable e tipo DOUBLE em Business_Settings
O sistema SHALL armazenar `investment_rate` e `profit_margin` como colunas `DOUBLE` nullable em `Business_Settings`, e SHALL garantir a migração de DBs existentes removendo a constraint NOT NULL e convertendo o tipo de `DECIMAL(5,4)` para `DOUBLE`.

**Nota técnica:** O tipo `DECIMAL(5,4)` não é lido corretamente como `f64` pelo duckdb-rs 1.4.4 (o scale não é aplicado, retornando sempre 0). `DOUBLE` mapeia diretamente para `f64`.

#### Scenario: DB novo com campos vazios
- **WHEN** o app é iniciado em um banco novo e o usuário cria a empresa sem preencher as taxas
- **THEN** os campos `investment_rate` e `profit_margin` são armazenados como NULL

#### Scenario: Migração de DB existente — remoção de NOT NULL
- **WHEN** o app é iniciado em um banco existente que possui `investment_rate NOT NULL`
- **THEN** a constraint NOT NULL é removida sem afetar os dados existentes

#### Scenario: Migração de DB existente — tipo DOUBLE
- **WHEN** o app é iniciado em um banco existente com `investment_rate DECIMAL(5,4)`
- **THEN** o tipo da coluna é convertido para `DOUBLE` sem afetar os dados existentes

### Requirement: Taxas null tratadas como zero na precificação
O sistema SHALL tratar `investment_rate = NULL` e `profit_margin = NULL` como `0.0` ao calcular `PricingRates`, sem erro.

#### Scenario: Cálculo com taxas null
- **WHEN** `get_pricing_rates` é invocado e `investment_rate` ou `profit_margin` é NULL no banco
- **THEN** o sistema retorna `investment_rate = 0.0` e/ou `profit_margin = 0.0` em `PricingRates`

### Requirement: Derivação de taxas de precificação

O sistema SHALL calcular CV% e DF% para a fórmula de markup a partir de `Financial_Entries` e do faturamento dos últimos 12 meses em `Sales`.

#### Scenario: Taxas calculadas com histórico
- **WHEN** o comando `get_pricing_rates` é invocado e existem registros em `Sales` dos últimos 12 meses
- **THEN** o sistema retorna `cv`, `df`, `investment_rate` e `profit_margin` onde `cv = Σ amount (is_fixed=false) / faturamento_ref` e `df = Σ amount (type='expense', is_fixed=true) / faturamento_ref`

#### Scenario: Taxas zeradas sem histórico de vendas
- **WHEN** o comando `get_pricing_rates` é invocado e não existem vendas nos últimos 12 meses
- **THEN** o sistema retorna `cv = 0` e `df = 0` com flag `no_sales_history = true`
