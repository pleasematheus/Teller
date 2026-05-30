## MODIFIED Requirements

### Requirement: Registro de compra multi-item

O sistema SHALL registrar uma compra com múltiplos itens em uma única operação atômica, criando automaticamente as movimentações de entrada em estoque para cada item com `creates_stock = true`. Para cada item, o sistema SHALL **fixar a taxa de perda do insumo**: calcula a taxa sugerida (ver "Snapshot da taxa de perda na compra"), aceita um override por item, grava o valor escolhido em `Purchase_Items.loss_rate` e atualiza `Products.expected_loss_rate` do insumo.

#### Scenario: Compra com frete e múltiplos itens
- **WHEN** o comando `create_purchase` é invocado com `freight >= 0` e ao menos um item válido
- **THEN** o sistema insere em `Purchases`, insere cada item em `Purchase_Items` (incluindo `loss_rate`), cria `Stock_Movements (movement_type='in', motive='compra')` para cada item com `creates_stock = true`, e atualiza `expected_loss_rate` do produto — tudo em uma transação

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

### Requirement: Snapshot da taxa de perda na compra

O sistema SHALL sugerir, por item de compra, a taxa de perda medida do insumo (`get_product_loss_rate`, sobre unidades resolvidas em 12 meses) e SHALL permitir que o operador sobrescreva esse valor antes de salvar. O valor escolhido SHALL ser congelado em `Purchase_Items.loss_rate` e propagado para `Products.expected_loss_rate` do insumo, tornando-se a perda usada na precificação até a próxima compra.

#### Scenario: Sugestão com histórico disponível
- **WHEN** um item de compra é adicionado para um insumo com unidades resolvidas nos últimos 12 meses
- **THEN** o formulário pré-popula o campo de perda do item com a taxa medida

#### Scenario: Sugestão sem histórico
- **WHEN** o insumo não tem unidades resolvidas nos últimos 12 meses
- **THEN** a perda sugerida é 0 e o operador pode informar um valor manual

#### Scenario: Override manual
- **WHEN** o operador edita a taxa de perda de um item antes de salvar a compra
- **THEN** o valor informado prevalece sobre a sugestão e é o que se grava em `Purchase_Items.loss_rate` e `Products.expected_loss_rate`

#### Scenario: Snapshot por compra preservado
- **WHEN** uma compra é salva com perda fixada
- **THEN** `Purchase_Items.loss_rate` retém o valor daquela compra para auditoria, independentemente de compras futuras alterarem `Products.expected_loss_rate`
