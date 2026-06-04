## MODIFIED Requirements

### Requirement: Registro de compra multi-item

O sistema SHALL registrar uma compra com múltiplos itens como **rascunho** em uma única operação, criando o registro em `Purchases` com `status = 'open'` e inserindo cada item em `Purchase_Items`, **sem** criar movimentações de estoque e **sem** alterar custo, preço ou taxa de perda dos produtos. A aplicação desses efeitos ocorre apenas no fechamento (ver "Fechamento de compra"). Cada item SHALL referenciar um produto **existente** (via `product_id`) **ou** ser um **produto novo**, caso em que `product_id` é nulo e o item guarda os campos pendentes `new_product_name`, `new_product_unit`, `new_product_description` (opcional) e `new_product_sellable`; exatamente uma das duas formas SHALL estar presente por item, e nenhum produto SHALL ser criado em `Products` enquanto a compra está em rascunho. Cada item SHALL guardar `quantity`, `unit_cost`, o fator de conversão opcional `unit_qty` e o `final_price` pretendido. A conversão de unidade de compra para unidade de estoque SHALL usar o fator único `unit_qty` (`stock = quantity * unit_qty`, ou `quantity` quando nulo). A seleção de produto de cada linha SHALL usar o combobox de seleção de produto (ver capability `product-search`), que oferece a ação de criar produto novo. Cada linha SHALL expor um campo de preço de venda editável (`final_price`) pré-preenchido pelo preço sugerido (`computePrice`); o operador SHALL poder sobrescrevê-lo. Edição de itens e do frete SHALL ser permitida apenas enquanto `status = 'open'`.

#### Scenario: Criação de rascunho de compra
- **WHEN** o comando `create_purchase` é invocado com `freight >= 0` e ao menos um item válido
- **THEN** o sistema insere em `Purchases` com `status = 'open'` e insere cada item em `Purchase_Items`, sem criar `Stock_Movements` e sem alterar nenhum produto

#### Scenario: Rascunho com item de produto novo
- **WHEN** um item do rascunho não tem `product_id` e traz `new_product_name`, `new_product_unit`, `new_product_description` (opcional) e `new_product_sellable`
- **THEN** o sistema persiste esses campos pendentes em `Purchase_Items` com `product_id` nulo e NÃO cria registro em `Products`

#### Scenario: Item sem produto existente nem dados de produto novo
- **WHEN** um item não tem `product_id` e também não tem `new_product_name`/`new_product_unit`
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

#### Scenario: Conversão por fator único
- **WHEN** um item de compra tem `unit_qty` preenchido
- **THEN** a quantidade considerada para estoque no fechamento é `quantity * unit_qty`

#### Scenario: Sem conversão de unidade
- **WHEN** um item de compra tem `unit_qty` nulo
- **THEN** a quantidade considerada para estoque no fechamento é igual à `quantity` do item

#### Scenario: Edição bloqueada após fechamento
- **WHEN** uma compra tem `status = 'closed'` e o operador tenta alterar seus itens ou frete
- **THEN** o sistema rejeita a alteração

### Requirement: Fechamento de compra

O sistema SHALL fornecer o comando `close_purchase(purchase_id)` que, em uma única transação, aplica os efeitos de uma compra em rascunho e marca `status = 'closed'`. Antes de aplicar os efeitos de cada item, se o item for de **produto novo** (`product_id` nulo com campos `new_product_*`), o sistema SHALL criar o produto em `Products` (validando nome e unidade ativa em `Units_Of_Measure`, com `cost_price=0`, `markup=0`, `final_price=0`, `profit_margin=0`, `expected_loss_rate=0` iniciais), fazer backfill de `Purchase_Items.product_id` com o id gerado, e então tratá-lo como um item de produto existente. Para cada item com `creates_stock = true`, o sistema SHALL: fixar a taxa de perda (gravando em `Purchase_Items.loss_rate` e em `Products.expected_loss_rate`/`loss_mode`); recalcular `Products.cost_price` como média ponderada por quantidade incluindo o frete rateado proporcional ao valor da linha; persistir `Products.final_price` e derivar `markup`/`profit_margin` como snapshot; e criar `Stock_Movements ('in','compra')` com quantidade `quantity * unit_qty` (ou `quantity` quando `unit_qty` é nulo). Uma compra já `closed` não SHALL ser fechada novamente.

#### Scenario: Fechamento cria o produto novo
- **WHEN** `close_purchase` é invocado para um rascunho que contém um item de produto novo válido
- **THEN** o sistema cria o produto em `Products`, faz backfill de `product_id` no item, e aplica custo/perda/preço/movimentação de estoque para esse produto, tudo na mesma transação

#### Scenario: Produto novo com unidade inválida
- **WHEN** um item de produto novo tem `new_product_unit` inexistente ou inativa em `Units_Of_Measure`
- **THEN** o fechamento retorna erro, a transação sofre rollback e o `status` permanece `open`

#### Scenario: Produto novo não persiste quando o fechamento falha
- **WHEN** o fechamento de um rascunho com item de produto novo falha em um passo posterior
- **THEN** a transação sofre rollback e nenhum produto permanece em `Products`

#### Scenario: Fechamento aplica estoque e custo
- **WHEN** `close_purchase` é invocado para uma compra `open` com itens válidos
- **THEN** o sistema cria `Stock_Movements ('in','compra')` para cada item, atualiza perda, `cost_price`, `final_price`, `markup` e `profit_margin` dos produtos, e marca `status = 'closed'` — tudo em uma transação
