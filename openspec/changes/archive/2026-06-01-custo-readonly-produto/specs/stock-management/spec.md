## MODIFIED Requirements

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

## ADDED Requirements

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
