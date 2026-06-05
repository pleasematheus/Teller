## ADDED Requirements

### Requirement: Quantidade de item de compra respeita o flag inteiro da unidade

O campo de quantidade de cada linha de item de compra SHALL respeitar o flag `integer` da unidade do produto da linha. Quando a unidade do produto tem `integer = true`, o formulário SHALL usar passo inteiro (`step="1"`) e SHALL bloquear o envio de uma `quantity` com casas decimais, exibindo erro de validação. Quando a unidade tem `integer = false`, casas decimais SHALL ser permitidas. Quando a unidade do produto não for encontrada na lista de unidades, o sistema SHALL tratá-la como inteira (bloqueia decimais), coerente com o default.

#### Scenario: Unidade inteira bloqueia decimal
- **WHEN** o produto da linha usa uma unidade com `integer = true` e o operador informa `quantity = 2,5`
- **THEN** o formulário exibe erro de validação e não envia o item

#### Scenario: Unidade inteira aceita inteiro
- **WHEN** o produto da linha usa uma unidade com `integer = true` e o operador informa `quantity = 3`
- **THEN** o item é aceito com `quantity = 3`

#### Scenario: Unidade decimal aceita decimal
- **WHEN** o produto da linha usa uma unidade com `integer = false` e o operador informa `quantity = 2,5`
- **THEN** o item é aceito com `quantity = 2,5`

#### Scenario: Unidade ausente assume inteiro
- **WHEN** a unidade do produto da linha não consta na lista de unidades carregada
- **THEN** o campo de quantidade usa passo inteiro e bloqueia o envio de quantidade decimal
