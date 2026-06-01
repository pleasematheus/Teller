## MODIFIED Requirements

### Requirement: Histórico de movimentações por produto

O sistema SHALL retornar o histórico completo de movimentações de um produto específico, ordenado do mais recente para o mais antigo. Cada movimentação SHALL incluir o `sale_id` da venda que a originou quando a movimentação for de venda, e `null` para movimentações sem venda associada (compra, perda, ajuste).

#### Scenario: Produto com histórico
- **WHEN** o comando `list_stock_movements` é invocado com `product_id` válido
- **THEN** o sistema retorna todos os registros de `Stock_Movements` daquele produto com `id`, `quantity`, `movement_type`, `motive`, `created_at` e `sale_id`, ordenados por `created_at DESC`

#### Scenario: Produto sem histórico
- **WHEN** o produto não possui movimentações
- **THEN** o sistema retorna lista vazia sem erro

#### Scenario: Movimentação de venda traz o ID da venda
- **WHEN** uma movimentação foi criada pelo fechamento de uma venda (`movement_type='out'`, `motive='venda'`)
- **THEN** o registro retornado tem `sale_id` igual ao `id` da venda que a originou

#### Scenario: Movimentação sem venda associada
- **WHEN** uma movimentação tem `motive` em (`compra`, `perda`, `ajuste`)
- **THEN** o registro retornado tem `sale_id = null`

#### Scenario: Exibição do ID da venda na tela de estoque
- **WHEN** o histórico de movimentações é exibido e uma movimentação possui `sale_id`
- **THEN** a tela exibe o identificador da venda para essa linha; movimentações com `sale_id = null` exibem o campo vazio

### Requirement: Baixa de estoque no fechamento da venda registra a venda de origem

O sistema SHALL registrar, em cada movimentação de saída criada no fechamento de uma venda, o `sale_id` da venda que a originou, tanto para produtos sem componentes quanto para insumos consumidos via `Product_Components`.

#### Scenario: Baixa de produto sem componentes
- **WHEN** uma venda é fechada e um produto vendido não possui componentes em `Product_Components`
- **THEN** a `Stock_Movement (movement_type='out', motive='venda')` criada para esse produto tem `sale_id` igual ao `id` da venda

#### Scenario: Baixa de insumos via BOM
- **WHEN** uma venda é fechada e um produto vendido possui componentes em `Product_Components`
- **THEN** cada `Stock_Movement (movement_type='out', motive='venda')` criada para os insumos consumidos tem `sale_id` igual ao `id` da venda
