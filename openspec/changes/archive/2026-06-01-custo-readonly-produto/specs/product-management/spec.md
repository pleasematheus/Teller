## MODIFIED Requirements

### Requirement: Cadastro de produto

O sistema SHALL permitir criar um novo produto com nome, descrição opcional, flags `active` e `sellable`, unidade de medida e **preço final de venda**. A **taxa de perda não é informada no cadastro**: um produto novo nasce com `expected_loss_rate = 0` e `loss_mode = manual`, valores fixados posteriormente pela compra do insumo (ver stock-management). O **`cost_price` também não é informado no cadastro**: um produto novo nasce com `cost_price = 0` e o custo passa a ser fixado pela compra do insumo (média ponderada — ver stock-management). Para produto **com componentes** (ficha técnica), o `cost_price` é **derivado** dos insumos e não é editável. O comando `create_product` NÃO SHALL aceitar `cost_price` como entrada do formulário. O sistema SHALL derivar `markup` e `profit_margin` a partir do preço final informado e do CMV aplicável, e persistir os três campos.

#### Scenario: Cadastro de insumo sem custo manual
- **WHEN** o usuário submete um produto sem componentes com nome não-vazio, `unit` válido e `final_price >= 0`
- **THEN** o sistema insere o produto com `cost_price = 0`, `expected_loss_rate = 0`, `loss_mode = manual`, e `markup`/`profit_margin` derivados do preço final, e retorna sucesso

#### Scenario: Cadastro de produto com componentes (custo derivado)
- **WHEN** o usuário submete um produto com componentes cadastrados
- **THEN** o `cost_price`/CMV é derivado da ficha técnica (ver requirement "CMV derivado da ficha técnica") e nenhum custo manual é considerado

#### Scenario: Nome vazio
- **WHEN** o usuário submete nome em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Custo não é entrada do cadastro
- **WHEN** o formulário de cadastro de produto é renderizado
- **THEN** não há campo editável de custo; o produto é criado com `cost_price = 0` e o custo passa a ser fixado na primeira compra do insumo (ver stock-management)

#### Scenario: Taxa de perda não é entrada do cadastro
- **WHEN** o formulário de cadastro de produto é renderizado
- **THEN** não há campo editável de taxa de perda; o produto é criado com `expected_loss_rate = 0` e a perda passa a ser fixada na primeira compra do insumo (ver stock-management)

### Requirement: Edição de produto

O sistema SHALL permitir atualizar os dados de um produto existente incluindo `unit` e `final_price`, recalculando `markup` e `profit_margin` derivados do preço final ao salvar. A edição **NÃO altera** `cost_price`, `expected_loss_rate` nem `loss_mode`: esses campos são propriedade do evento de compra (ver stock-management) e aparecem no formulário apenas em leitura. O comando `update_product` NÃO SHALL aceitar `cost_price` como entrada.

#### Scenario: Edição com dados válidos
- **WHEN** o usuário edita um produto existente e submete dados válidos incluindo `unit` e `final_price >= CMV`
- **THEN** o sistema atualiza o registro em `Products` com os novos valores, recalcula `markup` e `profit_margin` derivados do `final_price`, e atualiza `updated_at`, sem alterar `cost_price`

#### Scenario: Edição não altera o custo fixado
- **WHEN** o usuário salva uma edição de produto
- **THEN** `cost_price` permanece com o valor fixado pela última compra do insumo, sem ser sobrescrito pelo salvamento do produto

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

O sistema SHALL permitir que o operador informe o **preço final** e SHALL calcular e exibir em tempo real o CMV e a **Margem de lucro derivada** conforme o usuário edita `final_price`, usando o `cost_price` **persistido** (somente leitura) e a `expected_loss_rate` **persistida** (somente leitura) e as taxas de `get_pricing_rates` (cujo `cv` já inclui a taxa de cartão embutida). O sistema NÃO SHALL exibir a porcentagem de markup no formulário (o `markup` continua sendo derivado e persistido, apenas não é mostrado). O preço de venda não SHALL ser recalculado automaticamente a partir do histórico da empresa.

#### Scenario: Cálculo com histórico de vendas disponível
- **WHEN** o formulário é aberto para um novo produto e existem vendas nos últimos 12 meses
- **THEN** CV% é derivado de (despesas variáveis de `Financial_Entries` + taxas de cartão pagas de `Sale_Payments`) ÷ faturamento dos últimos 12 meses; DF% é derivado das despesas fixas ÷ faturamento; o campo `final_price` é pré-populado com um **preço sugerido** derivado da margem realizada (`get_realized_margin`)

#### Scenario: Cálculo sem histórico de vendas
- **WHEN** o formulário é aberto e não existem vendas nos últimos 12 meses
- **THEN** CV% = 0, DF% = 0 e o preço sugerido recai sobre o preço de equilíbrio (margem zero); o formulário exibe aviso de que as taxas não puderam ser calculadas

#### Scenario: Atualização em tempo real
- **WHEN** o usuário altera `final_price`
- **THEN** o CMV e a Margem de lucro derivada são recalculados instantaneamente no cliente sem nova requisição, usando o `cost_price` e a `expected_loss_rate` persistidos e mantendo o `final_price` informado pelo operador como entrada estável

#### Scenario: Markup não exibido no formulário
- **WHEN** o formulário de produto é renderizado em qualquer estado
- **THEN** a porcentagem de markup não é exibida; apenas o custo, o CMV e a margem derivada aparecem como leitura

#### Scenario: Custo exibido somente leitura
- **WHEN** o formulário de um produto sem componentes é renderizado
- **THEN** o custo é exibido como leitura (valor de `cost_price`) sem campo editável; em produto novo (sem compra) exibe texto indicando que o custo é definido pelas compras do insumo

#### Scenario: Taxa de perda exibida somente leitura
- **WHEN** o formulário de um produto sem componentes é renderizado
- **THEN** a taxa de perda é exibida como leitura (valor de `expected_loss_rate`) com o rótulo do modo (`auto`/`manual`), sem campo editável

### Requirement: Validação dinâmica do campo de preço final

O sistema SHALL restringir o valor mínimo aceito no campo de preço final do formulário de produto ao CMV atual (`cost_price / (1 − expected_loss_rate)`). Como `cost_price` e `expected_loss_rate` são valores persistidos somente leitura no formulário, o mínimo é constante durante a edição (para produto com ficha técnica, o CMV derivado define o mínimo).

#### Scenario: Mínimo do campo derivado do CMV atual
- **WHEN** o formulário de produto é renderizado com `cost_price` e `expected_loss_rate` persistidos
- **THEN** o atributo de valor mínimo do campo de preço final corresponde ao CMV calculado

#### Scenario: Mínimo constante durante a edição
- **WHEN** o operador edita apenas o preço final sem que o custo persistido mude
- **THEN** o valor mínimo do campo permanece igual ao CMV derivado do `cost_price` e `expected_loss_rate` persistidos
