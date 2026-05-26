## ADDED Requirements

### Requirement: Verificação de cadastro de empresa
O sistema SHALL verificar se existe ao menos um registro em `Business_Settings` para determinar se o cadastro de empresa já foi realizado.

#### Scenario: Empresa não cadastrada
- **WHEN** a tabela `Business_Settings` não contém registros
- **THEN** o sistema retorna `false` indicando que o cadastro de empresa é necessário

#### Scenario: Empresa já cadastrada
- **WHEN** a tabela `Business_Settings` contém ao menos um registro
- **THEN** o sistema retorna `true` indicando que o cadastro já foi realizado

### Requirement: Cadastro de empresa
O sistema SHALL permitir registrar os dados da empresa com nome, taxa de investimento e margem de lucro, armazenando taxas como frações decimais (0–1).

#### Scenario: Cadastro com dados válidos
- **WHEN** o usuário submete nome não-vazio, taxa de investimento e margem de lucro válidos
- **THEN** o sistema insere um registro em `Business_Settings` e retorna sucesso

#### Scenario: Nome vazio
- **WHEN** o usuário submete nome em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Taxa fora do intervalo
- **WHEN** o usuário submete `investment_rate` ou `profit_margin` fora do intervalo 0–100%
- **THEN** o sistema retorna erro de validação sem criar o registro
