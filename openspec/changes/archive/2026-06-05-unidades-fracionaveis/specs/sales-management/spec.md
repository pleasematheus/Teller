## ADDED Requirements

### Requirement: Quantidade de item de venda respeita o flag inteiro da unidade

O campo de quantidade ao lado da busca de produto no pedido SHALL respeitar o flag `integer` da unidade do produto selecionado. Quando a unidade do produto tem `integer = true`, o formulário SHALL usar passo inteiro (`step="1"`) e SHALL bloquear a adição de um item com quantidade contendo casas decimais, exibindo erro de validação. Quando a unidade tem `integer = false`, casas decimais SHALL ser permitidas. Quando a unidade do produto não for encontrada na lista de unidades, o sistema SHALL tratá-la como inteira (bloqueia decimais), coerente com o default.

#### Scenario: Unidade inteira bloqueia decimal na venda
- **WHEN** o produto selecionado usa uma unidade com `integer = true` e o operador informa quantidade `1,5`
- **THEN** o formulário exibe erro de validação e não adiciona o item ao carrinho

#### Scenario: Unidade inteira aceita inteiro na venda
- **WHEN** o produto selecionado usa uma unidade com `integer = true` e o operador informa quantidade `2`
- **THEN** o item é adicionado ao carrinho com quantidade `2`

#### Scenario: Unidade decimal aceita decimal na venda
- **WHEN** o produto selecionado usa uma unidade com `integer = false` e o operador informa quantidade `1,5`
- **THEN** o item é adicionado ao carrinho com quantidade `1,5`

#### Scenario: Unidade ausente assume inteiro na venda
- **WHEN** a unidade do produto selecionado não consta na lista de unidades carregada
- **THEN** o campo de quantidade usa passo inteiro e bloqueia a adição de quantidade decimal
