## MODIFIED Requirements

### Requirement: Registro de compra multi-item

O sistema SHALL registrar uma compra com múltiplos itens em uma única operação atômica, criando automaticamente as movimentações de entrada em estoque para cada item com `creates_stock = true`. Para cada item, o sistema SHALL **fixar a taxa de perda do insumo**: calcula a taxa sugerida (ver "Snapshot da taxa de perda na compra"), aceita a escolha de modo (`auto`/`manual`) com override manual por item, grava o valor escolhido em `Purchase_Items.loss_rate` e atualiza `Products.expected_loss_rate` e `Products.loss_mode` do insumo.

#### Scenario: Compra com frete e múltiplos itens
- **WHEN** o comando `create_purchase` é invocado com `freight >= 0` e ao menos um item válido
- **THEN** o sistema insere em `Purchases`, insere cada item em `Purchase_Items` (incluindo `loss_rate`), cria `Stock_Movements (movement_type='in', motive='compra')` para cada item com `creates_stock = true`, e atualiza `expected_loss_rate` e `loss_mode` do produto — tudo em uma transação

#### Scenario: Conversão caixa para unidades
- **WHEN** um item de compra tem `box_qty` e `unit_qty` preenchidos
- **THEN** a quantidade movimentada em estoque é `quantity × (unit_qty ÷ box_qty)`

#### Scenario: Sem conversão de unidade
- **WHEN** um item de compra não tem `box_qty` e `unit_qty` preenchidos
- **THEN** a quantidade movimentada em estoque é igual à `quantity` do item

#### Scenario: Item com quantidade zero ou negativa
- **WHEN** qualquer item da compra tem `quantity <= 0`
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

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
