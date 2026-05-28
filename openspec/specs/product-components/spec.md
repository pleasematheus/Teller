## Purpose

Gerenciamento da ficha técnica de produtos compostos: define quais insumos (produtos com `sellable=false`) compõem uma unidade de um produto vendável, e em que quantidade.

## Requirements

### Requirement: Listagem de componentes de um produto

O sistema SHALL retornar todos os componentes cadastrados para um produto, com nome e quantidade de cada insumo.

#### Scenario: Produto com componentes
- **WHEN** o command `list_product_components` é invocado com `product_id` válido
- **THEN** o sistema retorna lista com `id`, `child_id`, `child_name`, `quantity`, `unit` de cada componente em `Product_Components` onde `parent_id = product_id`

#### Scenario: Produto sem componentes
- **WHEN** o command `list_product_components` é invocado com `product_id` de produto sem componentes
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Adição de componente à ficha técnica

O sistema SHALL permitir adicionar um insumo como componente de um produto vendável, com quantidade por unidade produzida.

#### Scenario: Adição válida
- **WHEN** o command `add_product_component` é invocado com `parent_id` de produto com `sellable=true`, `child_id` de produto com `sellable=false` e `quantity > 0`
- **THEN** o sistema insere o registro em `Product_Components` e retorna sucesso

#### Scenario: Produto pai não é vendável
- **WHEN** o command `add_product_component` é invocado com `parent_id` de produto com `sellable=false`
- **THEN** o sistema retorna erro de validação sem inserir o registro

#### Scenario: Componente não é insumo
- **WHEN** o command `add_product_component` é invocado com `child_id` de produto com `sellable=true`
- **THEN** o sistema retorna erro de validação sem inserir o registro

#### Scenario: Produto adicionando a si mesmo
- **WHEN** o command `add_product_component` é invocado com `parent_id = child_id`
- **THEN** o sistema retorna erro de validação sem inserir o registro

#### Scenario: Quantidade inválida
- **WHEN** o command `add_product_component` é invocado com `quantity <= 0`
- **THEN** o sistema retorna erro de validação sem inserir o registro

#### Scenario: Componente duplicado
- **WHEN** o command `add_product_component` é invocado com combinação `parent_id` + `child_id` já existente
- **THEN** o sistema retorna erro sem inserir o registro

### Requirement: Remoção de componente da ficha técnica

O sistema SHALL permitir remover um componente da ficha técnica de um produto.

#### Scenario: Remoção válida
- **WHEN** o command `remove_product_component` é invocado com `id` de componente existente
- **THEN** o sistema remove o registro de `Product_Components` e retorna sucesso

#### Scenario: Componente inexistente
- **WHEN** o command `remove_product_component` é invocado com `id` que não existe
- **THEN** o sistema retorna erro sem alterar nenhum registro

### Requirement: Atualização de quantidade de componente

O sistema SHALL permitir alterar a quantidade de um componente já cadastrado na ficha técnica.

#### Scenario: Atualização válida
- **WHEN** o command `update_product_component` é invocado com `id` de componente existente e `quantity > 0`
- **THEN** o sistema atualiza o campo `quantity` em `Product_Components` e retorna sucesso

#### Scenario: Quantidade inválida
- **WHEN** o command `update_product_component` é invocado com `quantity <= 0`
- **THEN** o sistema retorna erro de validação sem alterar o registro

### Requirement: Seção de componentes no formulário de produto

O sistema SHALL exibir a seção de ficha técnica no formulário de produto somente quando o produto é vendável e está sendo editado, permitindo adicionar, editar e remover componentes inline.

#### Scenario: Produto vendável em edição — seção visível
- **WHEN** o formulário de produto é exibido para edição com `sellable = true`
- **THEN** a seção "Componentes" é exibida com a lista de insumos já cadastrados e opção de adicionar novo

#### Scenario: Produto não-vendável — seção oculta
- **WHEN** o formulário de produto é exibido com `sellable = false`
- **THEN** a seção "Componentes" não é exibida

#### Scenario: Adição de componente pela UI
- **WHEN** o operador seleciona um insumo e informa a quantidade e confirma
- **THEN** o frontend invoca `add_product_component` e atualiza a lista sem recarregar o formulário inteiro

#### Scenario: Remoção de componente pela UI
- **WHEN** o operador clica em remover um componente da lista
- **THEN** o frontend invoca `remove_product_component` e remove o item da lista

#### Scenario: Edição de quantidade pela UI
- **WHEN** o operador edita a quantidade de um componente na lista e confirma
- **THEN** o frontend invoca `update_product_component` e reflete o novo valor
