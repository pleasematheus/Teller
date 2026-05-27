## Purpose

Controle de estoque de insumos da loja de fotos: consulta de saldo atual, registro de compras multi-item com movimentações automáticas, lançamento manual de perdas e ajustes, histórico de movimentações por produto e indicador de capacidade produtiva estimada.

## Requirements

### Requirement: Consulta de saldo de estoque

O sistema SHALL retornar o saldo atual de todos os insumos a partir da view `Stock`, incluindo produtos sem movimentações (saldo zero).

#### Scenario: Saldo com movimentações
- **WHEN** o comando `list_stock` é invocado
- **THEN** o sistema retorna `product_id`, `name`, `unit` e `current_quantity` para cada produto com movimentações, com `current_quantity = Σin - Σout`

#### Scenario: Produto sem movimentações
- **WHEN** um produto não possui registros em `Stock_Movements`
- **THEN** o sistema retorna `current_quantity = 0` para esse produto

### Requirement: Indicador de capacidade produtiva

O sistema SHALL calcular e exibir, para cada insumo, a quantidade estimada de produtos finais que podem ser produzidos com o saldo atual, com base na composição definida em `Product_Components`.

#### Scenario: Insumo com componentes cadastrados
- **WHEN** o módulo de estoque é aberto e o insumo faz parte de ao menos um produto via `Product_Components`
- **THEN** o sistema exibe "≈ N [produto]s possíveis" calculado como `current_quantity ÷ calculated_quantity`

#### Scenario: Insumo sem componentes cadastrados
- **WHEN** o insumo não participa de nenhum `Product_Components`
- **THEN** o indicador de capacidade não é exibido para esse insumo

### Requirement: Registro de compra multi-item

O sistema SHALL registrar uma compra com múltiplos itens em uma única operação atômica, criando automaticamente as movimentações de entrada em estoque para cada item com `creates_stock = true`.

#### Scenario: Compra com frete e múltiplos itens
- **WHEN** o comando `create_purchase` é invocado com `freight >= 0` e ao menos um item válido
- **THEN** o sistema insere em `Purchases`, insere cada item em `Purchase_Items` e cria `Stock_Movements (movement_type='in', motive='compra')` para cada item com `creates_stock = true`, tudo em uma transação

#### Scenario: Conversão caixa para unidades
- **WHEN** um item de compra tem `box_qty` e `unit_qty` preenchidos
- **THEN** a quantidade movimentada em estoque é `quantity × (unit_qty ÷ box_qty)`

#### Scenario: Sem conversão de unidade
- **WHEN** um item de compra não tem `box_qty` e `unit_qty` preenchidos
- **THEN** a quantidade movimentada em estoque é igual à `quantity` do item

#### Scenario: Item com quantidade zero ou negativa
- **WHEN** qualquer item da compra tem `quantity <= 0`
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

### Requirement: Lançamento manual de perda ou ajuste

O sistema SHALL permitir registrar uma movimentação manual de estoque para cobrir perdas (impressão errada) e correções de inventário.

#### Scenario: Registro de perda
- **WHEN** o comando `create_stock_adjustment` é invocado com `motive = 'perda'` e `quantity > 0`
- **THEN** o sistema cria `Stock_Movement (movement_type='out', motive='perda')` com a quantidade informada

#### Scenario: Ajuste positivo (entrada)
- **WHEN** o comando `create_stock_adjustment` é invocado com `motive = 'ajuste'` e `quantity > 0`
- **THEN** o sistema cria `Stock_Movement (movement_type='in', motive='ajuste')`

#### Scenario: Ajuste negativo (saída)
- **WHEN** o comando `create_stock_adjustment` é invocado com `motive = 'ajuste'` e `quantity < 0`
- **THEN** o sistema cria `Stock_Movement (movement_type='out', motive='ajuste')` com `abs(quantity)`

#### Scenario: Quantidade zero
- **WHEN** `quantity = 0` é informado
- **THEN** o sistema retorna erro de validação sem criar movimentação

### Requirement: Histórico de movimentações por produto

O sistema SHALL retornar o histórico completo de movimentações de um produto específico, ordenado do mais recente para o mais antigo.

#### Scenario: Produto com histórico
- **WHEN** o comando `list_stock_movements` é invocado com `product_id` válido
- **THEN** o sistema retorna todos os registros de `Stock_Movements` daquele produto com `id`, `quantity`, `movement_type`, `motive` e `created_at`, ordenados por `created_at DESC`

#### Scenario: Produto sem histórico
- **WHEN** o produto não possui movimentações
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Listagem de componentes de produto

O sistema SHALL retornar os componentes (insumos) de um produto específico com suas quantidades, para uso no cálculo do indicador de capacidade.

#### Scenario: Produto com componentes
- **WHEN** o comando `list_product_components` é invocado com `product_id` válido
- **THEN** o sistema retorna `child_id`, `child_name`, `quantity`, `calculated_quantity` e `use_calculated` para cada componente

#### Scenario: Produto sem componentes
- **WHEN** o produto não possui entradas em `Product_Components`
- **THEN** o sistema retorna lista vazia sem erro
