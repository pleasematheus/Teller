## ADDED Requirements

### Requirement: Exibição da quantidade em estoque no formulário de produto

O sistema SHALL exibir, na tela de edição de um produto existente (`ProductForm`), a quantidade atual em estoque do produto em modo **somente leitura**, reaproveitando o `current_quantity` retornado por `list_stock`. A exibição NÃO SHALL permitir editar o saldo nem disparar movimentações de estoque; o ajuste permanece exclusivo do módulo de Estoque. Na criação de um produto novo (sem `id` persistido) o campo de saldo NÃO SHALL ser exibido.

#### Scenario: Produto em edição exibe saldo atual
- **WHEN** o usuário abre para edição um produto que possui registro em `Stock`
- **THEN** o formulário exibe a quantidade atual em estoque (`current_quantity`) com a unidade do produto, em modo somente leitura

#### Scenario: Produto sem movimentação exibe saldo zero
- **WHEN** o usuário abre para edição um produto sem entradas em `Stock`
- **THEN** o formulário exibe quantidade `0` (LEFT JOIN com COALESCE), sem erro

#### Scenario: Criação de produto não exibe saldo
- **WHEN** o formulário é aberto para um produto novo (ainda não persistido)
- **THEN** o campo de quantidade em estoque não é exibido

#### Scenario: Saldo é somente leitura
- **WHEN** o formulário de produto exibe a quantidade em estoque
- **THEN** não há controle de edição, botão de ajuste ou submissão que altere o saldo a partir desta tela
