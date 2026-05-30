## MODIFIED Requirements

### Requirement: Cálculo automático de taxa de perda

O sistema SHALL calcular `expected_loss_rate` de um insumo a partir do histórico de `Stock_Movements` sobre **unidades resolvidas** (saídas por venda ou perda) numa janela de 12 meses, descartando unidades ainda em estoque. O cálculo SHALL servir como **sugestão fixada no evento de compra** (ver stock-management), não mais por ação "Calcular" no formulário de produto. O sistema NÃO recalcula `expected_loss_rate` automaticamente na leitura ou exibição.

#### Scenario: Cálculo com histórico resolvido disponível
- **WHEN** `get_product_loss_rate` é invocado para um insumo com saídas nos últimos 12 meses
- **THEN** o sistema retorna `Σ(quantity WHERE movement_type='out' AND motive='perda') / (Σ(quantity WHERE movement_type='out' AND motive='venda') + Σ(quantity WHERE movement_type='out' AND motive='perda'))`, considerando apenas movimentos com `created_at >= NOW() - INTERVAL 12 MONTHS`

#### Scenario: Estoque parado não dilui a taxa
- **WHEN** um insumo tem entradas (`in`) ainda não resolvidas em estoque
- **THEN** essas unidades não entram em nenhum lado da fração; a taxa reflete apenas venda e perda já ocorridas

#### Scenario: Cálculo sem unidades resolvidas
- **WHEN** o insumo não possui saídas por venda nem por perda nos últimos 12 meses
- **THEN** o sistema retorna 0 sem erro

#### Scenario: Valor congelado entre compras
- **WHEN** novos movimentos de perda são registrados após uma compra
- **THEN** `expected_loss_rate` permanece com o valor fixado na última compra até a próxima compra do insumo o refixar

### Requirement: Cadastro de produto

O sistema SHALL permitir criar um novo produto com nome, descrição opcional, flags `active` e `sellable`, taxa de perda, unidade de medida e **preço final de venda**. Para produto **sem componentes**, o operador informa o `cost_price`; para produto **com componentes** (ficha técnica), o `cost_price` é **derivado** dos insumos e não é editável. O sistema SHALL derivar `markup` e `profit_margin` a partir do preço final informado e do CMV aplicável, e persistir os três campos.

#### Scenario: Cadastro de produto sem componentes (custo próprio)
- **WHEN** o usuário submete um produto sem componentes com nome não-vazio, `cost_price >= 0`, `expected_loss_rate` entre 0 e 1, `unit` válido e `final_price >= CMV` (CMV = `cost_price / (1 − expected_loss_rate)`)
- **THEN** o sistema insere o produto com `markup` e `profit_margin` derivados do preço final, e retorna sucesso

#### Scenario: Cadastro de produto com componentes (custo derivado)
- **WHEN** o usuário submete um produto com componentes cadastrados
- **THEN** o `cost_price`/CMV é derivado da ficha técnica (ver requirement "CMV derivado da ficha técnica") e o campo de custo manual não é considerado

#### Scenario: Nome vazio
- **WHEN** o usuário submete nome em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Taxa de perda inválida
- **WHEN** o usuário submete `expected_loss_rate` fora do intervalo 0–1
- **THEN** o sistema retorna erro de validação sem criar o registro

## ADDED Requirements

### Requirement: CMV derivado da ficha técnica

O sistema SHALL derivar o CMV de um produto vendável **com componentes** a partir da ficha técnica, somando o custo efetivo de cada insumo (custo inflado pela própria perda) multiplicado pela quantidade do componente. Produto **sem componentes** mantém o CMV a partir do próprio `cost_price` e `expected_loss_rate`.

#### Scenario: Rollup com componentes
- **WHEN** o CMV de um produto com componentes é calculado
- **THEN** `CMV = Σ_componentes [ child.cost_price / (1 − child.expected_loss_rate) ] × component.quantity`

#### Scenario: Produto sem componentes usa custo próprio
- **WHEN** o produto não possui entradas em `Product_Components`
- **THEN** `CMV = cost_price / (1 − expected_loss_rate)` do próprio produto

#### Scenario: Componente com perda 100%
- **WHEN** algum insumo da ficha técnica tem `expected_loss_rate = 1`
- **THEN** o sistema sinaliza CMV indefinido para aquele componente (divisão por zero) e bloqueia a precificação com aviso, sem produzir um custo inválido

#### Scenario: Vendável sem ficha técnica cadastrada
- **WHEN** um produto vendável não tem componentes nem `cost_price` próprio significativo
- **THEN** o formulário exibe aviso de que o custo não pode ser derivado até a ficha técnica ser cadastrada
