## MODIFIED Requirements

### Requirement: Cadastro de produto

O sistema SHALL permitir criar um novo produto com nome, descrição opcional, flags `active` e `sellable`, unidade de medida e **preço final de venda**. A **taxa de perda não é informada no cadastro**: um produto novo nasce com `expected_loss_rate = 0` e `loss_mode = manual`, valores fixados posteriormente pela compra do insumo (ver stock-management). Para produto **sem componentes**, o operador informa o `cost_price`; para produto **com componentes** (ficha técnica), o `cost_price` é **derivado** dos insumos e não é editável. O sistema SHALL derivar `markup` e `profit_margin` a partir do preço final informado e do CMV aplicável, e persistir os três campos.

#### Scenario: Cadastro de produto sem componentes (custo próprio)
- **WHEN** o usuário submete um produto sem componentes com nome não-vazio, `cost_price >= 0`, `unit` válido e `final_price >= CMV` (CMV = `cost_price / (1 − expected_loss_rate)`, com `expected_loss_rate = 0` em produto novo)
- **THEN** o sistema insere o produto com `expected_loss_rate = 0`, `loss_mode = manual`, e `markup`/`profit_margin` derivados do preço final, e retorna sucesso

#### Scenario: Cadastro de produto com componentes (custo derivado)
- **WHEN** o usuário submete um produto com componentes cadastrados
- **THEN** o `cost_price`/CMV é derivado da ficha técnica (ver requirement "CMV derivado da ficha técnica") e o campo de custo manual não é considerado

#### Scenario: Nome vazio
- **WHEN** o usuário submete nome em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Taxa de perda não é entrada do cadastro
- **WHEN** o formulário de cadastro de produto é renderizado
- **THEN** não há campo editável de taxa de perda; o produto é criado com `expected_loss_rate = 0` e a perda passa a ser fixada na primeira compra do insumo (ver stock-management)

### Requirement: Edição de produto

O sistema SHALL permitir atualizar os dados de um produto existente incluindo `unit` e `final_price`, recalculando `markup` e `profit_margin` derivados do preço final ao salvar. A edição **NÃO altera** `expected_loss_rate` nem `loss_mode`: esses campos são propriedade do evento de compra (ver stock-management) e aparecem no formulário apenas em leitura.

#### Scenario: Edição com dados válidos
- **WHEN** o usuário edita um produto existente e submete dados válidos incluindo `unit` e `final_price >= CMV`
- **THEN** o sistema atualiza o registro em `Products` com os novos valores, recalcula `markup` e `profit_margin` derivados do `final_price`, e atualiza `updated_at`

#### Scenario: Edição não altera a perda fixada
- **WHEN** o usuário salva uma edição de produto
- **THEN** `expected_loss_rate` e `loss_mode` permanecem com o valor fixado pela última compra do insumo, sem serem sobrescritos pelo salvamento do produto

#### Scenario: Preço abaixo do custo na edição
- **WHEN** o usuário submete `final_price < CMV`
- **THEN** o sistema retorna erro de prejuízo sem alterar o registro

#### Scenario: Produto inexistente
- **WHEN** o comando `update_product` é invocado com `id` que não existe
- **THEN** o sistema retorna erro sem alterar nenhum registro

### Requirement: Precificação automática no formulário

O sistema SHALL permitir que o operador informe o **preço final** e SHALL calcular e exibir em tempo real o CMV e a **Margem de lucro derivada** conforme o usuário edita `cost_price` e `final_price`, usando a `expected_loss_rate` **persistida** (somente leitura) e as taxas de `get_pricing_rates` (cujo `cv` já inclui a taxa de cartão embutida). O sistema NÃO SHALL exibir a porcentagem de markup no formulário (o `markup` continua sendo derivado e persistido, apenas não é mostrado). O preço de venda não SHALL ser recalculado automaticamente a partir do histórico da empresa.

#### Scenario: Cálculo com histórico de vendas disponível
- **WHEN** o formulário é aberto para um novo produto e existem vendas nos últimos 12 meses
- **THEN** CV% é derivado de (despesas variáveis de `Financial_Entries` + taxas de cartão pagas de `Sale_Payments`) ÷ faturamento dos últimos 12 meses; DF% é derivado das despesas fixas ÷ faturamento; o campo `final_price` é pré-populado com um **preço sugerido** derivado da margem realizada (`get_realized_margin`)

#### Scenario: Cálculo sem histórico de vendas
- **WHEN** o formulário é aberto e não existem vendas nos últimos 12 meses
- **THEN** CV% = 0, DF% = 0 e o preço sugerido recai sobre o preço de equilíbrio (margem zero); o formulário exibe aviso de que as taxas não puderam ser calculadas

#### Scenario: Atualização em tempo real
- **WHEN** o usuário altera `cost_price` ou `final_price`
- **THEN** o CMV e a Margem de lucro derivada são recalculados instantaneamente no cliente sem nova requisição, usando a `expected_loss_rate` persistida e mantendo o `final_price` informado pelo operador como entrada estável

#### Scenario: Markup não exibido no formulário
- **WHEN** o formulário de produto é renderizado em qualquer estado
- **THEN** a porcentagem de markup não é exibida; apenas o custo, o CMV e a margem derivada aparecem como leitura

#### Scenario: Taxa de perda exibida somente leitura
- **WHEN** o formulário de um produto sem componentes é renderizado
- **THEN** a taxa de perda é exibida como leitura (valor de `expected_loss_rate`) com o rótulo do modo (`auto`/`manual`), sem campo editável

### Requirement: Validação dinâmica do campo de preço final

O sistema SHALL restringir dinamicamente o valor mínimo aceito no campo de preço final do formulário de produto ao CMV atual (`cost_price / (1 − expected_loss_rate)`), recalculado conforme `cost_price` muda. A `expected_loss_rate` é constante no formulário (valor persistido, somente leitura), então o mínimo varia apenas com o custo.

#### Scenario: Mínimo do campo derivado do CMV atual
- **WHEN** o formulário de produto é renderizado com `cost_price` e `expected_loss_rate`
- **THEN** o atributo de valor mínimo do campo de preço final corresponde ao CMV calculado

#### Scenario: Mínimo acompanha mudança de custo
- **WHEN** o usuário altera `cost_price`
- **THEN** o valor mínimo do campo de preço final é recalculado para refletir o novo CMV, mantendo a `expected_loss_rate` persistida

### Requirement: Persistência do modo da taxa de perda

O sistema SHALL persistir o modo da taxa de perda (`loss_mode`, com valores `manual` ou `auto`) por produto, com default `manual`. O modo é **definido no evento de compra** (ver stock-management), não no formulário de produto, e é **restaurado em modo somente leitura** ao abrir o produto para edição. Salvar o produto não altera `loss_mode` nem `expected_loss_rate`.

#### Scenario: Modo definido na compra
- **WHEN** uma compra fixa a perda de um insumo com o modo `auto` selecionado no item
- **THEN** o sistema grava `loss_mode = auto` no registro do produto

#### Scenario: Restaurar modo somente leitura na edição
- **WHEN** o usuário abre para edição um produto salvo com `loss_mode = auto`
- **THEN** o formulário exibe o modo `auto` em leitura ao lado da taxa de perda, sem controle de edição

#### Scenario: Default manual
- **WHEN** um produto é criado sem `loss_mode` informado ou é lido de um registro legado sem o campo
- **THEN** o sistema assume `loss_mode = manual`

#### Scenario: Modo inválido rejeitado
- **WHEN** o comando de compra recebe um `loss_mode` diferente de `manual` ou `auto`
- **THEN** o sistema rejeita com erro de validação

#### Scenario: Salvar produto não altera o modo
- **WHEN** o usuário salva uma edição de produto
- **THEN** `loss_mode` permanece com o valor fixado pela última compra, sem recálculo nem sobrescrita

### Requirement: Cálculo automático de taxa de perda

O sistema SHALL calcular `expected_loss_rate` de um insumo a partir do histórico de `Stock_Movements` sobre **unidades resolvidas** (saídas por venda ou perda) numa janela de 12 meses, descartando unidades ainda em estoque. A medição SHALL considerar o **motivo-base** do movimento, ignorando qualquer observação anexada ao motivo (ex.: `"perda - Impressão com defeito"` conta como `perda`). O cálculo SHALL servir como **sugestão fixada no evento de compra** (ver stock-management), não por ação "Calcular" no formulário de produto. O sistema NÃO recalcula `expected_loss_rate` automaticamente na leitura ou exibição.

#### Scenario: Cálculo com histórico resolvido disponível
- **WHEN** `get_product_loss_rate` é invocado para um insumo com saídas nos últimos 12 meses
- **THEN** o sistema retorna `Σ(quantity de saídas cujo motivo-base é 'perda') / (Σ(quantity de saídas cujo motivo-base é 'venda') + Σ(quantity de saídas cujo motivo-base é 'perda'))`, considerando apenas movimentos com `created_at >= NOW() - INTERVAL 12 MONTHS`

#### Scenario: Perda registrada com observação é contabilizada
- **WHEN** uma perda é registrada com observação e gravada como `motive = "perda - <observação>"`
- **THEN** a medição a contabiliza como perda (pelo motivo-base), sem ser ignorada por causa da observação

#### Scenario: Estoque parado não dilui a taxa
- **WHEN** um insumo tem entradas (`in`) ainda não resolvidas em estoque
- **THEN** essas unidades não entram em nenhum lado da fração; a taxa reflete apenas venda e perda já ocorridas

#### Scenario: Cálculo sem unidades resolvidas
- **WHEN** o insumo não possui saídas por venda nem por perda nos últimos 12 meses
- **THEN** o sistema retorna 0 sem erro

#### Scenario: Valor congelado entre compras
- **WHEN** novos movimentos de perda são registrados após uma compra
- **THEN** `expected_loss_rate` permanece com o valor fixado na última compra até a próxima compra do insumo o refixar
