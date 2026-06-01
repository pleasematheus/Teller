## Purpose

Listagem, cadastro e edição de produtos com distinção entre insumos e produtos vendáveis. O operador informa o **preço final** diretamente; a margem de lucro é derivada pelo sistema via inversão da fórmula de markup divisor (`margin = 1 − cv − df − ir − CMV/preço`). O CMV atua como piso de custo para evitar precificação no prejuízo.

## Requirements

### Requirement: Listagem de produtos

O sistema SHALL retornar todos os produtos cadastrados com seus dados de precificação, ordenados por nome, incluindo os campos `sellable`, `unit`, `profit_margin` e `loss_mode` para uso no cliente.

#### Scenario: Lista com produtos existentes
- **WHEN** o comando `list_products` é invocado
- **THEN** o sistema retorna todos os registros de `Products` com campos: id, name, description, active, sellable, cost_price, markup, final_price, expected_loss_rate, unit, profit_margin, **loss_mode**

#### Scenario: Lista vazia
- **WHEN** nenhum produto está cadastrado
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Filtro de produtos por tipo

O sistema SHALL permitir filtrar a lista de produtos por [Todos / Vendáveis / Insumos] no cliente, sem nova requisição ao backend.

#### Scenario: Filtro "Vendáveis"
- **WHEN** o usuário seleciona o filtro "Vendáveis"
- **THEN** a lista exibe apenas produtos com `sellable = true`

#### Scenario: Filtro "Insumos"
- **WHEN** o usuário seleciona o filtro "Insumos"
- **THEN** a lista exibe apenas produtos com `sellable = false`

#### Scenario: Filtro "Todos"
- **WHEN** o usuário seleciona o filtro "Todos"
- **THEN** a lista exibe todos os produtos independente de `sellable`

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

### Requirement: Taxa de cartão embutida na precificação

O sistema SHALL embutir a taxa de cartão no preço dos produtos como custo variável, somando as taxas de cartão pagas ao numerador do `cv` em `get_pricing_rates`, de modo que o preço financie a maquininha (repasse difuso, com denominador `taxas ÷ faturamento`).

#### Scenario: Taxa de cartão somada ao cv com histórico
- **WHEN** `get_pricing_rates` é invocado e há pagamentos em cartão nos últimos 12 meses
- **THEN** `cv = (despesas variáveis + SUM(amount_paid − net_amount) dos pagamentos em cartão dos últimos 12 meses) ÷ faturamento dos últimos 12 meses`

#### Scenario: Fallback sem histórico de cartão
- **WHEN** não há pagamentos em cartão nos últimos 12 meses, mas existem taxas de cartão configuradas
- **THEN** a parcela de cartão do `cv` usa a maior taxa configurada (`MAX(fee_percent)`) como referência conservadora, até que exista histórico realizado

#### Scenario: Sem cartão e sem taxas configuradas
- **WHEN** não há pagamentos em cartão nem taxas de cartão configuradas
- **THEN** a parcela de cartão do `cv` é 0, sem erro

#### Scenario: Preço final cobre a taxa de cartão
- **WHEN** um produto é precificado com a taxa de cartão presente no `cv`
- **THEN** o `final_price` resultante (CMV × markup) cobre, no agregado, as taxas de cartão retidas, de forma que a margem-alvo se mantém para o mix de pagamento histórico

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

### Requirement: Cálculo automático de taxa de perda

O sistema SHALL calcular `expected_loss_rate` de um insumo a partir do histórico de `Stock_Movements` sobre **unidades resolvidas** (saídas por venda ou perda) numa janela de 12 meses, descartando unidades ainda em estoque. A medição SHALL considerar o **motivo-base** do movimento, ignorando qualquer observação anexada ao motivo (ex.: `"perda - Impressão com defeito"` conta como `perda`). O cálculo SHALL servir como **sugestão fixada no evento de compra** (ver stock-management), não mais por ação "Calcular" no formulário de produto. O sistema NÃO recalcula `expected_loss_rate` automaticamente na leitura ou exibição.

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

### Requirement: Margem de lucro por produto

O sistema SHALL armazenar uma `profit_margin` individual por produto, **derivada do preço final no momento do salvamento** e usada como leitura analítica, não mais como entrada da precificação.

#### Scenario: profit_margin derivada do preço final
- **WHEN** um produto é salvo com `final_price` válido
- **THEN** o sistema persiste `profit_margin = 1 − cv − df − investment_rate − CMV/final_price` como snapshot do instante do salvamento

#### Scenario: Preço sugerido pré-populado na criação
- **WHEN** o formulário de criação de produto é aberto
- **THEN** o campo de preço final é pré-populado com um preço sugerido derivado de `get_realized_margin` (média dos últimos 12 meses de vendas fechadas)

#### Scenario: Sugestão sem histórico de vendas
- **WHEN** `get_realized_margin` retorna 0 por ausência de vendas fechadas nos últimos 12 meses
- **THEN** o preço sugerido recai sobre o preço de equilíbrio (margem zero) e o formulário exibe aviso de que não há histórico disponível

#### Scenario: Preço editável
- **WHEN** o operador altera manualmente o valor do preço final no formulário
- **THEN** o valor informado pelo operador prevalece sobre a sugestão automática e a margem derivada é recalculada

### Requirement: Piso de custo e margem derivada do preço final

O sistema SHALL derivar a margem de lucro do preço final via `computeMarginFromPrice`, que SHALL retornar `cmv`, `markup_percent`, `profit_margin` (podendo ser negativa), `min_price = CMV`, `break_even_price`, `below_cost` e `feasible`. O sistema SHALL tratar o CMV como piso de custo: preço abaixo do CMV é prejuízo e SHALL ser bloqueado; preço entre o CMV e o preço de equilíbrio gera margem negativa e SHALL ser comunicado como aviso.

#### Scenario: Preço acima do equilíbrio gera margem positiva
- **WHEN** `final_price > break_even_price` e `1 − cv − df − investment_rate > 0`
- **THEN** `computeMarginFromPrice` retorna `feasible: true`, `below_cost: false` e `profit_margin > 0`

#### Scenario: Preço abaixo do custo sinaliza prejuízo
- **WHEN** `final_price < CMV`
- **THEN** `computeMarginFromPrice` retorna `below_cost: true` e `feasible: false`, e o formulário exibe aviso de prejuízo e bloqueia o salvamento

#### Scenario: Preço entre custo e equilíbrio gera margem negativa
- **WHEN** `CMV <= final_price < break_even_price`
- **THEN** `computeMarginFromPrice` retorna `profit_margin < 0` com `below_cost: false`, e o formulário exibe aviso de margem negativa indicando o preço de equilíbrio

#### Scenario: Taxas consomem todo o preço
- **WHEN** `cv + df + investment_rate >= 1`
- **THEN** `computeMarginFromPrice` retorna `feasible: false` com `break_even_price` indefinido (nenhum preço positivo gera margem ≥ 0), e o formulário comunica que as taxas já consomem o preço

### Requirement: Validação dinâmica do campo de preço final

O sistema SHALL restringir o valor mínimo aceito no campo de preço final do formulário de produto ao CMV atual (`cost_price / (1 − expected_loss_rate)`). Como `cost_price` e `expected_loss_rate` são valores persistidos somente leitura no formulário, o mínimo é constante durante a edição (para produto com ficha técnica, o CMV derivado define o mínimo).

#### Scenario: Mínimo do campo derivado do CMV atual
- **WHEN** o formulário de produto é renderizado com `cost_price` e `expected_loss_rate` persistidos
- **THEN** o atributo de valor mínimo do campo de preço final corresponde ao CMV calculado

#### Scenario: Mínimo constante durante a edição
- **WHEN** o operador edita apenas o preço final sem que o custo persistido mude
- **THEN** o valor mínimo do campo permanece igual ao CMV derivado do `cost_price` e `expected_loss_rate` persistidos

### Requirement: Cálculo de margem realizada

O sistema SHALL calcular a margem de lucro média realizada dos últimos 12 meses a partir das vendas fechadas.

#### Scenario: Cálculo com vendas disponíveis
- **WHEN** o command `get_realized_margin` é invocado
- **THEN** o sistema retorna a média de `(sp.discounted_price - cmv × sp.quantity) / sp.discounted_price` para todos os itens de `Sales_Products` onde `Sales.status = 'closed'` e `Sales.created_at >= NOW() - INTERVAL 12 MONTHS`, onde `cmv = cost_price / (1 - expected_loss_rate)`

#### Scenario: Sem vendas nos últimos 12 meses
- **WHEN** não há vendas fechadas nos últimos 12 meses
- **THEN** o command retorna `0` sem erro

### Requirement: Exibição da quantidade em estoque no formulário de produto

O sistema SHALL exibir, na tela de edição de um produto existente (`ProductForm`), a quantidade atual em estoque do produto em modo **somente leitura**, reaproveitando o `current_quantity` retornado por `list_stock`. A exibição NÃO SHALL permitir editar o saldo nem disparar movimentações de estoque; o ajuste permanece exclusivo do módulo de Estoque. Na criação de um produto novo (sem `id` persistido) o campo de saldo NÃO SHALL ser exibido.

#### Scenario: Produto em edição exibe saldo atual
- **WHEN** o usuário abre para edição um produto que possui registro em `Stock`
- **THEN** o formulário exibe a quantidade atual em estoque (`current_quantity`) com a unidade do produto, em modo somente leitura

#### Scenario: Produto sem movimentação exibe saldo zero
- **WHEN** o usuário abre para edição um produto sem entradas em `Stock`
- **THEN** o formulário exibe quantidade `0` (LEFT JOIN com COALESCE), sem erro

#### Scenario: Criação de produto não exibe saldo
- **WHEN** o formulário é aberto para um produto novo (ainda não persistido)
- **THEN** o campo de quantidade em estoque não é exibido

#### Scenario: Saldo é somente leitura
- **WHEN** o formulário de produto exibe a quantidade em estoque
- **THEN** não há controle de edição, botão de ajuste ou submissão que altere o saldo a partir desta tela

