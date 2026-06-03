## MODIFIED Requirements

### Requirement: Registro de compra multi-item

O sistema SHALL registrar uma compra com múltiplos itens em uma única operação atômica, criando automaticamente as movimentações de entrada em estoque para cada item com `creates_stock = true`. Para cada item, o sistema SHALL **fixar a taxa de perda do insumo**: calcula a taxa sugerida (ver "Snapshot da taxa de perda na compra"), aceita a escolha de modo (`auto`/`manual`) com override manual por item, grava o valor escolhido em `Purchase_Items.loss_rate` e atualiza `Products.expected_loss_rate` e `Products.loss_mode` do insumo. Para cada item, o sistema SHALL também **fixar o custo do insumo** recalculando `Products.cost_price` como **média ponderada por quantidade** (ver "Custo médio ponderado do insumo na compra"). A seleção de produto de cada linha SHALL usar o combobox de seleção de produto (ver capability `product-search`) no lugar do `<select>` nativo. Cada linha de item SHALL expor um campo de **preço de venda editável** (`final_price`, o que o cliente paga), pré-preenchido com o **preço sugerido** pela fórmula de precificação existente (`computePrice`) a partir do custo efetivo do lote (mercadoria + frete rateado, por unidade de estoque), da taxa de perda do insumo e das taxas globais; o operador SHALL poder sobrescrever esse valor. Ao salvar, `create_purchase` SHALL persistir o `final_price` informado em `Products` e derivar `markup` e `profit_margin` como snapshot do instante do salvamento (`cmv = cost_price/(1 − expected_loss_rate)`, `markup = final_price/cmv`, `profit_margin = 1 − cv − df − investment_rate − cmv/final_price`), usando o `cost_price` resultante da compra.

#### Scenario: Compra com frete e múltiplos itens
- **WHEN** o comando `create_purchase` é invocado com `freight >= 0` e ao menos um item válido
- **THEN** o sistema insere em `Purchases`, insere cada item em `Purchase_Items` (incluindo `loss_rate`), cria `Stock_Movements (movement_type='in', motive='compra')` para cada item com `creates_stock = true`, atualiza `expected_loss_rate` e `loss_mode` do produto, atualiza `cost_price` pela média ponderada e atualiza `final_price`, `markup` e `profit_margin` do produto — tudo em uma transação

#### Scenario: Preço de venda sugerido por item
- **WHEN** o operador define produto, quantidade e custo de uma linha (e o frete da compra)
- **THEN** o sistema exibe no campo de preço de venda da linha o preço sugerido por `computePrice`, calculado sobre o custo efetivo do lote (incluindo a parcela de frete rateada), a taxa de perda do insumo e as taxas globais

#### Scenario: Operador sobrescreve o preço de venda
- **WHEN** o operador edita o campo de preço de venda de uma linha
- **THEN** o valor informado passa a ser o `final_price` enviado no payload de `create_purchase` para aquele item, ignorando o preço sugerido

#### Scenario: Persistência do preço de venda no produto
- **WHEN** uma compra é salva com `final_price` por item
- **THEN** o `Products.final_price` do insumo passa a ser o valor informado e `Products.markup`/`Products.profit_margin` são derivados do `final_price` e do `cost_price` resultante, conforme a regra de snapshot de precificação

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

### Requirement: Custo médio ponderado do insumo na compra

O sistema SHALL recalcular o `Products.cost_price` de cada insumo comprado como **média ponderada por quantidade em estoque**, tornando a compra a única fonte do custo do insumo (espelhando o que já ocorre com `expected_loss_rate`). O custo total do lote de um item SHALL ser `quantity × unit_cost` somado à **parcela de frete rateada** para aquele item. O frete SHALL ser rateado entre os itens proporcionalmente ao valor de cada linha: `freight_share_i = freight × (quantity_i × unit_cost_i) ÷ Σ(quantity × unit_cost)`; quando o somatório dos valores de linha é zero, nenhuma parcela de frete é rateada. O custo unitário do lote SHALL ser o custo total do lote (mercadoria + frete rateado) dividido pela quantidade efetivamente movimentada em estoque (após conversão caixa→unidade). O novo custo SHALL ser `(saldo_atual × cost_price_atual + qtd_estoque_lote × custo_unitário_lote) / (saldo_atual + qtd_estoque_lote)`, usando o saldo de estoque anterior à entrada do lote. O `freight` SHALL continuar gravado em `Purchases.total_cost`.

#### Scenario: Primeira compra de um insumo sem estoque
- **WHEN** uma compra fixa o custo de um insumo cujo saldo de estoque anterior é 0
- **THEN** `cost_price` passa a ser o custo unitário de estoque do lote (`(quantity × unit_cost + frete_rateado) ÷ qtd_estoque_lote`), sem diluição por saldo anterior

#### Scenario: Compra subsequente com estoque existente
- **WHEN** uma compra fixa o custo de um insumo que já tem saldo de estoque com um `cost_price` vigente
- **THEN** `cost_price` passa a ser a média ponderada `(saldo_atual × cost_price_atual + qtd_estoque_lote × custo_unitário_lote) / (saldo_atual + qtd_estoque_lote)`

#### Scenario: Frete rateado entra no custo do insumo
- **WHEN** uma compra com `freight > 0` e mais de um item é registrada
- **THEN** cada insumo recebe uma parcela de frete proporcional ao valor da sua linha (`freight × valor_linha ÷ subtotal`), somada ao custo do lote antes do cálculo da média ponderada, e o `freight` total permanece gravado em `Purchases.total_cost`

#### Scenario: Frete não rateado quando o subtotal é zero
- **WHEN** uma compra com `freight > 0` é registrada e o somatório dos valores de linha é zero
- **THEN** nenhuma parcela de frete é somada ao `cost_price` de qualquer insumo e o `freight` permanece apenas em `Purchases.total_cost`

#### Scenario: Conversão caixa para unidade no custo unitário
- **WHEN** um item de compra tem `box_qty` e `unit_qty` preenchidos (a quantidade de estoque difere da quantidade comprada)
- **THEN** o custo unitário de estoque do lote é `(quantity × unit_cost + frete_rateado) ÷ (quantity × (unit_qty ÷ box_qty))`, mantendo a média ponderada coerente com a unidade de estoque

#### Scenario: Custo congelado entre compras
- **WHEN** movimentos de venda ou perda ocorrem após uma compra, sem nova compra do insumo
- **THEN** `cost_price` permanece com o valor fixado na última compra até a próxima compra do insumo o refixar
