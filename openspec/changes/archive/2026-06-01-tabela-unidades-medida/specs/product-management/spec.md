## MODIFIED Requirements

### Requirement: Cadastro de produto

O sistema SHALL permitir criar um novo produto com nome, descrição opcional, flags `active` e `sellable`, unidade de medida e **preço final de venda**. A unidade informada SHALL ser **válida**, isto é, existir e estar ativa em `Units_Of_Measure`; o sistema SHALL rejeitar `unit` que não exista na tabela. A **taxa de perda não é informada no cadastro**: um produto novo nasce com `expected_loss_rate = 0` e `loss_mode = manual`, valores fixados posteriormente pela compra do insumo (ver stock-management). O **`cost_price` também não é informado no cadastro**: um produto novo nasce com `cost_price = 0` e o custo passa a ser fixado pela compra do insumo (média ponderada — ver stock-management). Para produto **com componentes** (ficha técnica), o `cost_price` é **derivado** dos insumos e não é editável. O comando `create_product` NÃO SHALL aceitar `cost_price` como entrada do formulário. O sistema SHALL derivar `markup` e `profit_margin` a partir do preço final informado e do CMV aplicável, e persistir os três campos.

#### Scenario: Cadastro de insumo sem custo manual
- **WHEN** o usuário submete um produto sem componentes com nome não-vazio, `unit` válido e `final_price >= 0`
- **THEN** o sistema insere o produto com `cost_price = 0`, `expected_loss_rate = 0`, `loss_mode = manual`, e `markup`/`profit_margin` derivados do preço final, e retorna sucesso

#### Scenario: Cadastro de produto com componentes (custo derivado)
- **WHEN** o usuário submete um produto com componentes cadastrados
- **THEN** o `cost_price`/CMV é derivado da ficha técnica (ver requirement "CMV derivado da ficha técnica") e nenhum custo manual é considerado

#### Scenario: Nome vazio
- **WHEN** o usuário submete nome em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Unidade inexistente
- **WHEN** o usuário submete um `unit` que não existe em `Units_Of_Measure`
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Custo não é entrada do cadastro
- **WHEN** o formulário de cadastro de produto é renderizado
- **THEN** não há campo editável de custo; o produto é criado com `cost_price = 0` e o custo passa a ser fixado na primeira compra do insumo (ver stock-management)

#### Scenario: Taxa de perda não é entrada do cadastro
- **WHEN** o formulário de cadastro de produto é renderizado
- **THEN** não há campo editável de taxa de perda; o produto é criado com `expected_loss_rate = 0` e a perda passa a ser fixada na primeira compra do insumo (ver stock-management)

### Requirement: Edição de produto

O sistema SHALL permitir atualizar os dados de um produto existente incluindo `unit` e `final_price`, recalculando `markup` e `profit_margin` derivados do preço final ao salvar. A unidade informada SHALL ser válida (existir em `Units_Of_Measure`); o sistema SHALL rejeitar `unit` inexistente. A edição **NÃO altera** `cost_price`, `expected_loss_rate` nem `loss_mode`: esses campos são propriedade do evento de compra (ver stock-management) e aparecem no formulário apenas em leitura. O comando `update_product` NÃO SHALL aceitar `cost_price` como entrada.

#### Scenario: Edição com dados válidos
- **WHEN** o usuário edita um produto existente e submete dados válidos incluindo `unit` válido e `final_price >= CMV`
- **THEN** o sistema atualiza o registro em `Products` com os novos valores, recalcula `markup` e `profit_margin` derivados do `final_price`, e atualiza `updated_at`, sem alterar `cost_price`

#### Scenario: Edição com unidade inexistente
- **WHEN** o usuário submete um `unit` que não existe em `Units_Of_Measure`
- **THEN** o sistema retorna erro de validação sem alterar o registro

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

## ADDED Requirements

### Requirement: Seletor de unidade dinâmico no formulário de produto

O formulário de produto SHALL popular o seletor de unidade de medida a partir de `list_units_of_measure`, exibindo apenas unidades ativas, em vez de uma lista fixa de opções embutidas no código. Cada opção SHALL exibir o nome e a sigla da unidade.

#### Scenario: Seletor carrega unidades ativas
- **WHEN** o formulário de produto é aberto
- **THEN** o seletor de unidade lista as unidades ativas retornadas por `list_units_of_measure`, mostrando nome e sigla

#### Scenario: Unidade desativada não aparece
- **WHEN** uma unidade está com `active = false`
- **THEN** ela não é oferecida como opção no seletor, embora produtos existentes que a utilizam mantenham o valor

#### Scenario: Produto em edição preserva unidade atual
- **WHEN** um produto é aberto para edição
- **THEN** o seletor apresenta a unidade atual do produto como valor selecionado
