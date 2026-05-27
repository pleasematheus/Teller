## MODIFIED Requirements

### Requirement: Listagem de produtos
O sistema SHALL retornar todos os produtos cadastrados com seus dados de precificação, ordenados por nome, incluindo os campos `sellable` e `unit` para uso no cliente.

#### Scenario: Lista com produtos existentes
- **WHEN** o comando `list_products` é invocado
- **THEN** o sistema retorna todos os registros de `Products` com campos: id, name, description, active, sellable, cost_price, markup, final_price, expected_loss_rate, **unit**

#### Scenario: Lista vazia
- **WHEN** nenhum produto está cadastrado
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Cadastro de produto
O sistema SHALL permitir criar um novo produto com nome, descrição opcional, flags `active` e `sellable`, custo, taxa de perda e **unidade de medida**.

#### Scenario: Cadastro com dados válidos
- **WHEN** o usuário submete nome não-vazio, `cost_price >= 0`, `expected_loss_rate` entre 0 e 1 e `unit` válido
- **THEN** o sistema insere o produto em `Products` com `markup` e `final_price` calculados e retorna sucesso

#### Scenario: Nome vazio
- **WHEN** o usuário submete nome em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Taxa de perda inválida
- **WHEN** o usuário submete `expected_loss_rate` fora do intervalo 0–1
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Taxa de perda = 1 (100%)
- **WHEN** o usuário submete `expected_loss_rate = 1`
- **THEN** o sistema retorna erro de validação (divisão por zero no CMV)

### Requirement: Edição de produto
O sistema SHALL permitir atualizar os dados de um produto existente incluindo `unit`, recalculando `markup` e `final_price` ao salvar.

#### Scenario: Edição com dados válidos
- **WHEN** o usuário edita um produto existente e submete dados válidos incluindo `unit`
- **THEN** o sistema atualiza o registro em `Products` com os novos valores e `updated_at` atualizado

#### Scenario: Produto inexistente
- **WHEN** o comando `update_product` é invocado com `id` que não existe
- **THEN** o sistema retorna erro sem alterar nenhum registro
