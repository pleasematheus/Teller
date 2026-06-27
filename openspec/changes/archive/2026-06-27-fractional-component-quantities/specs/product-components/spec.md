## MODIFIED Requirements

### Requirement: Adição de componente à ficha técnica

O sistema SHALL permitir adicionar um insumo como componente de um produto vendável, com quantidade fracionária por unidade produzida. A quantidade SHALL ser um número decimal positivo; o frontend SHALL rejeitar decimais quando a unidade do insumo for inteira (`Units_Of_Measure.integer = true`).

#### Scenario: Adição válida com quantidade inteira
- **WHEN** o command `add_product_component` é invocado com `parent_id` de produto com `sellable=true`, `child_id` de produto com `sellable=false` e `quantity = 1`
- **THEN** o sistema insere o registro em `Product_Components` com `quantity` decimal e retorna sucesso

#### Scenario: Adição válida com quantidade fracionária
- **WHEN** o command `add_product_component` é invocado com insumo de unidade fracionária (ex mL) e `quantity = 2.5`
- **THEN** o sistema insere o registro em `Product_Components` preservando o valor decimal e retorna sucesso

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

#### Scenario: Decimal em unidade inteira rejeitado na UI
- **WHEN** o operador informa quantidade decimal para um insumo cuja unidade tem `integer = true`
- **THEN** o frontend exibe erro de validação e não invoca `add_product_component`

#### Scenario: Componente duplicado
- **WHEN** o command `add_product_component` é invocado com combinação `parent_id` + `child_id` já existente
- **THEN** o sistema retorna erro sem inserir o registro

### Requirement: Atualização de quantidade de componente

O sistema SHALL permitir alterar a quantidade de um componente já cadastrado na ficha técnica para um valor decimal positivo, com a mesma validação de unidade inteira aplicada no frontend.

#### Scenario: Atualização válida com fração
- **WHEN** o command `update_product_component` é invocado com `id` de componente existente e `quantity = 0.5`
- **THEN** o sistema atualiza o campo `quantity` em `Product_Components` preservando o decimal e retorna sucesso

#### Scenario: Quantidade inválida
- **WHEN** o command `update_product_component` é invocado com `quantity <= 0`
- **THEN** o sistema retorna erro de validação sem alterar o registro

#### Scenario: Decimal em unidade inteira rejeitado na UI
- **WHEN** o operador edita a quantidade para um valor decimal em insumo de unidade inteira
- **THEN** o frontend exibe erro de validação e não invoca `update_product_component`

### Requirement: Seção de componentes no formulário de produto

O sistema SHALL exibir a seção de ficha técnica no formulário de produto somente quando o produto é vendável e está sendo editado, permitindo adicionar, editar e remover componentes inline. O input de quantidade SHALL ajustar `step`/`min` conforme o flag `integer` da unidade do insumo e validar o valor com `validateQuantity` antes de chamar o backend.

#### Scenario: Produto vendável em edição — seção visível
- **WHEN** o formulário de produto é exibido para edição com `sellable = true`
- **THEN** a seção "Componentes" é exibida com a lista de insumos já cadastrados e opção de adicionar novo

#### Scenario: Produto não-vendável — seção oculta
- **WHEN** o formulário de produto é exibido com `sellable = false`
- **THEN** a seção "Componentes" não é exibida

#### Scenario: Adição de componente pela UI com unidade fracionária
- **WHEN** o operador seleciona um insumo de unidade fracionária, informa quantidade decimal válida e confirma
- **THEN** o input aceita decimais (`step` fracionário), `validateQuantity` aprova, o frontend invoca `add_product_component` e atualiza a lista sem recarregar o formulário inteiro

#### Scenario: Remoção de componente pela UI
- **WHEN** o operador clica em remover um componente da lista
- **THEN** o frontend invoca `remove_product_component` e remove o item da lista

#### Scenario: Edição de quantidade pela UI
- **WHEN** o operador edita a quantidade de um componente na lista para valor válido e confirma
- **THEN** o frontend invoca `update_product_component` e reflete o novo valor
