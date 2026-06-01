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

O sistema SHALL registrar uma compra com múltiplos itens em uma única operação atômica, criando automaticamente as movimentações de entrada em estoque para cada item com `creates_stock = true`. Para cada item, o sistema SHALL **fixar a taxa de perda do insumo**: calcula a taxa sugerida (ver "Snapshot da taxa de perda na compra"), aceita a escolha de modo (`auto`/`manual`) com override manual por item, grava o valor escolhido em `Purchase_Items.loss_rate` e atualiza `Products.expected_loss_rate` e `Products.loss_mode` do insumo. Para cada item, o sistema SHALL também **fixar o custo do insumo** recalculando `Products.cost_price` como **média ponderada por quantidade** (ver "Custo médio ponderado do insumo na compra"). A seleção de produto de cada linha SHALL usar o combobox de seleção de produto (ver capability `product-search`) no lugar do `<select>` nativo.

#### Scenario: Compra com frete e múltiplos itens
- **WHEN** o comando `create_purchase` é invocado com `freight >= 0` e ao menos um item válido
- **THEN** o sistema insere em `Purchases`, insere cada item em `Purchase_Items` (incluindo `loss_rate`), cria `Stock_Movements (movement_type='in', motive='compra')` para cada item com `creates_stock = true`, atualiza `expected_loss_rate` e `loss_mode` do produto, e atualiza `cost_price` do produto pela média ponderada — tudo em uma transação

#### Scenario: Seleção de produto via combobox
- **WHEN** o operador abre o seletor de produto de uma linha de item e escolhe um produto no combobox
- **THEN** a linha passa a exibir o nome do produto escolhido, o `productId` da linha é definido e o sistema dispara `get_product_loss_rate` para preencher a perda sugerida daquela linha

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

### Requirement: Listagem de componentes de produto

O sistema SHALL retornar os componentes (insumos) de um produto específico com suas quantidades, para uso no cálculo do indicador de capacidade.

#### Scenario: Produto com componentes
- **WHEN** o comando `list_product_components` é invocado com `product_id` válido
- **THEN** o sistema retorna `child_id`, `child_name`, `quantity`, `calculated_quantity` e `use_calculated` para cada componente

#### Scenario: Produto sem componentes
- **WHEN** o produto não possui entradas em `Product_Components`
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Baixa de estoque no fechamento da venda registra a venda de origem

O sistema SHALL registrar, em cada movimentação de saída criada no fechamento de uma venda, o `sale_id` da venda que a originou, tanto para produtos sem componentes quanto para insumos consumidos via `Product_Components`.

#### Scenario: Baixa de produto sem componentes
- **WHEN** uma venda é fechada e um produto vendido não possui componentes em `Product_Components`
- **THEN** a `Stock_Movement (movement_type='out', motive='venda')` criada para esse produto tem `sale_id` igual ao `id` da venda

#### Scenario: Baixa de insumos via BOM
- **WHEN** uma venda é fechada e um produto vendido possui componentes em `Product_Components`
- **THEN** cada `Stock_Movement (movement_type='out', motive='venda')` criada para os insumos consumidos tem `sale_id` igual ao `id` da venda

### Requirement: Snapshot da taxa de perda na compra

O sistema SHALL oferecer, por item de compra, a escolha explícita do modo da perda: **Automático** (usa a taxa medida do insumo via `get_product_loss_rate`, exibida em leitura) ou **Manual** (o operador informa o valor). O valor efetivo SHALL ser congelado em `Purchase_Items.loss_rate` e propagado para `Products.expected_loss_rate`, e o modo escolhido SHALL ser gravado em `Products.loss_mode`, tornando-se a perda e o modo usados na precificação até a próxima compra.

#### Scenario: Modo automático usa a taxa medida
- **WHEN** um item de compra é salvo com o modo `auto`
- **THEN** a perda congelada é a taxa medida (`get_product_loss_rate` no momento da compra) e o produto recebe `loss_mode = auto`

#### Scenario: Modo manual usa o valor informado
- **WHEN** um item de compra é salvo com o modo `manual` e um valor de perda informado
- **THEN** o valor informado é o que se grava em `Purchase_Items.loss_rate` e `Products.expected_loss_rate`, e o produto recebe `loss_mode = manual`

#### Scenario: Sugestão com histórico disponível
- **WHEN** um item de compra é adicionado para um insumo com unidades resolvidas nos últimos 12 meses
- **THEN** o formulário pré-popula a taxa medida do item (exibida em leitura no modo automático)

#### Scenario: Sugestão sem histórico
- **WHEN** o insumo não tem unidades resolvidas nos últimos 12 meses
- **THEN** a perda sugerida é 0 e o operador pode alternar para manual e informar um valor

#### Scenario: Snapshot por compra preservado
- **WHEN** uma compra é salva com perda fixada
- **THEN** `Purchase_Items.loss_rate` retém o valor daquela compra para auditoria, independentemente de compras futuras alterarem `Products.expected_loss_rate`

### Requirement: Custo médio ponderado do insumo na compra

O sistema SHALL recalcular o `Products.cost_price` de cada insumo comprado como **média ponderada por quantidade em estoque**, tornando a compra a única fonte do custo do insumo (espelhando o que já ocorre com `expected_loss_rate`). O custo unitário do lote SHALL ser o custo por **unidade de estoque** do item (custo total do lote dividido pela quantidade efetivamente movimentada em estoque, após conversão caixa→unidade). O **frete fica fora** do `cost_price`, permanecendo apenas em `Purchases.total_cost`. O novo custo SHALL ser `(saldo_atual × cost_price_atual + qtd_estoque_lote × custo_unitário_lote) / (saldo_atual + qtd_estoque_lote)`, usando o saldo de estoque anterior à entrada do lote.

#### Scenario: Primeira compra de um insumo sem estoque
- **WHEN** uma compra fixa o custo de um insumo cujo saldo de estoque anterior é 0
- **THEN** `cost_price` passa a ser o custo unitário de estoque do lote (`quantity × unit_cost ÷ qtd_estoque_lote`), sem diluição por saldo anterior

#### Scenario: Compra subsequente com estoque existente
- **WHEN** uma compra fixa o custo de um insumo que já tem saldo de estoque com um `cost_price` vigente
- **THEN** `cost_price` passa a ser a média ponderada `(saldo_atual × cost_price_atual + qtd_estoque_lote × custo_unitário_lote) / (saldo_atual + qtd_estoque_lote)`

#### Scenario: Frete não entra no custo do insumo
- **WHEN** uma compra com `freight > 0` é registrada
- **THEN** o `freight` é gravado em `Purchases.total_cost` mas NÃO é rateado nem somado ao `cost_price` de nenhum insumo

#### Scenario: Conversão caixa para unidade no custo unitário
- **WHEN** um item de compra tem `box_qty` e `unit_qty` preenchidos (a quantidade de estoque difere da quantidade comprada)
- **THEN** o custo unitário de estoque do lote é `(quantity × unit_cost) ÷ (quantity × (unit_qty ÷ box_qty))`, mantendo a média ponderada coerente com a unidade de estoque

#### Scenario: Custo congelado entre compras
- **WHEN** movimentos de venda ou perda ocorrem após uma compra, sem nova compra do insumo
- **THEN** `cost_price` permanece com o valor fixado na última compra até a próxima compra do insumo o refixar
