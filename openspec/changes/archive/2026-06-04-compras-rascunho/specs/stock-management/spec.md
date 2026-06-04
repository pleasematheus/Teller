## MODIFIED Requirements

### Requirement: Registro de compra multi-item

O sistema SHALL registrar uma compra com múltiplos itens como **rascunho** em uma única operação, criando o registro em `Purchases` com `status = 'open'` e inserindo cada item em `Purchase_Items`, **sem** criar movimentações de estoque e **sem** alterar custo, preço ou taxa de perda dos produtos. A aplicação desses efeitos ocorre apenas no fechamento (ver "Fechamento de compra"). Cada item SHALL guardar `quantity`, `unit_cost`, o fator de conversão opcional `unit_qty` e o `final_price` pretendido. A conversão de unidade de compra para unidade de estoque SHALL usar o fator único `unit_qty`: a quantidade que entrará em estoque ao fechar é `quantity * unit_qty` quando `unit_qty` está preenchido, ou `quantity` quando `unit_qty` é nulo. A seleção de produto de cada linha SHALL usar o combobox de seleção de produto (ver capability `product-search`). Cada linha SHALL expor um campo de preço de venda editável (`final_price`) pré-preenchido pelo preço sugerido (`computePrice`); o operador SHALL poder sobrescrevê-lo. Edição de itens e do frete SHALL ser permitida apenas enquanto `status = 'open'`.

#### Scenario: Criação de rascunho de compra
- **WHEN** o comando `create_purchase` é invocado com `freight >= 0` e ao menos um item válido
- **THEN** o sistema insere em `Purchases` com `status = 'open'` e insere cada item em `Purchase_Items`, sem criar `Stock_Movements` e sem alterar `cost_price`, `final_price`, `markup`, `profit_margin`, `expected_loss_rate` ou `loss_mode` de nenhum produto

#### Scenario: Item com quantidade zero ou negativa
- **WHEN** qualquer item da compra tem `quantity <= 0`
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

#### Scenario: Conversão por fator único
- **WHEN** um item de compra tem `unit_qty` preenchido
- **THEN** a quantidade considerada para estoque no fechamento é `quantity * unit_qty`

#### Scenario: Sem conversão de unidade
- **WHEN** um item de compra tem `unit_qty` nulo
- **THEN** a quantidade considerada para estoque no fechamento é igual à `quantity` do item

#### Scenario: Seleção de produto via combobox
- **WHEN** o operador abre o seletor de produto de uma linha de item e escolhe um produto no combobox
- **THEN** a linha passa a exibir o nome do produto escolhido, o `productId` da linha é definido e o sistema dispara `get_product_loss_rate` para preencher a perda sugerida daquela linha

#### Scenario: Edição bloqueada após fechamento
- **WHEN** uma compra tem `status = 'closed'` e o operador tenta alterar seus itens ou frete
- **THEN** o sistema rejeita a alteração

## ADDED Requirements

### Requirement: Fechamento de compra

O sistema SHALL fornecer o comando `close_purchase(purchase_id)` que, em uma única transação, aplica os efeitos de uma compra em rascunho e marca `status = 'closed'`. Para cada item com `creates_stock = true`, o sistema SHALL: fixar a taxa de perda do insumo (modo `auto`/`manual` com override por item, gravando em `Purchase_Items.loss_rate` e atualizando `Products.expected_loss_rate` e `Products.loss_mode`); recalcular `Products.cost_price` como média ponderada por quantidade incluindo o frete rateado proporcional ao valor da linha; persistir `Products.final_price` e derivar `markup` e `profit_margin` como snapshot (`cmv = cost_price/(1 − expected_loss_rate)`, `markup = final_price/cmv`, `profit_margin = 1 − cv − df − investment_rate − cmv/final_price`); e criar `Stock_Movements (movement_type='in', motive='compra')` com quantidade `quantity * unit_qty` (ou `quantity` quando `unit_qty` é nulo). O fechamento SHALL ser idempotente em relação ao estado: uma compra já `closed` não SHALL ser fechada novamente.

#### Scenario: Fechamento aplica estoque e custo
- **WHEN** `close_purchase` é invocado para uma compra `open` com itens válidos
- **THEN** o sistema cria `Stock_Movements ('in','compra')` para cada item, atualiza `expected_loss_rate`, `loss_mode`, `cost_price` (média ponderada), `final_price`, `markup` e `profit_margin` dos produtos, e marca `Purchases.status = 'closed'` — tudo em uma transação

#### Scenario: Frete rateado entra no custo
- **WHEN** uma compra com `freight > 0` e múltiplos itens é fechada
- **THEN** cada `cost_price` recalculado inclui a parcela de frete proporcional ao valor da linha (`freight * line_value / subtotal`)

#### Scenario: Tentativa de fechar compra já fechada
- **WHEN** `close_purchase` é invocado para uma compra cujo `status` já é `closed`
- **THEN** o sistema retorna erro sem aplicar efeitos duplicados

#### Scenario: Falha no fechamento faz rollback
- **WHEN** qualquer passo do fechamento falha
- **THEN** a transação sofre rollback, nenhuma movimentação é criada e o `status` permanece `open`

### Requirement: Status da compra na listagem

O sistema SHALL permitir listar compras filtrando por `status`, expondo o `status` de cada compra para a interface diferenciar rascunhos de compras fechadas.

#### Scenario: Listagem filtrada por status
- **WHEN** `list_purchases` é invocado com um `status` (`open` ou `closed`)
- **THEN** o sistema retorna apenas as compras naquele status, incluindo o campo `status` em cada registro
