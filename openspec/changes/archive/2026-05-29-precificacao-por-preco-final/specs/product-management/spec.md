## MODIFIED Requirements

### Requirement: Cadastro de produto

O sistema SHALL permitir criar um novo produto com nome, descrição opcional, flags `active` e `sellable`, custo, taxa de perda, unidade de medida e **preço final de venda**. O sistema SHALL derivar `markup` e `profit_margin` a partir do preço final informado e persistir os três campos.

#### Scenario: Cadastro com dados válidos
- **WHEN** o usuário submete nome não-vazio, `cost_price >= 0`, `expected_loss_rate` entre 0 e 1, `unit` válido e `final_price >= CMV`
- **THEN** o sistema insere o produto em `Products` com `markup = final_price / CMV` e `profit_margin = 1 − cv − df − investment_rate − CMV/final_price` derivados do preço final, e retorna sucesso

#### Scenario: Preço abaixo do custo
- **WHEN** o usuário submete `final_price < CMV` (CMV = `cost_price / (1 − expected_loss_rate)`)
- **THEN** o sistema retorna erro de prejuízo (preço abaixo do custo) sem criar o registro

#### Scenario: Nome vazio
- **WHEN** o usuário submete nome em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Taxa de perda inválida
- **WHEN** o usuário submete `expected_loss_rate` fora do intervalo 0–1
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Taxa de perda = 1 (100%)
- **WHEN** o usuário submete `expected_loss_rate = 1`
- **THEN** o sistema retorna erro de validação (divisão por zero no CMV)

### Requirement: Edição de produto

O sistema SHALL permitir atualizar os dados de um produto existente incluindo `unit` e `final_price`, recalculando `markup` e `profit_margin` derivados do preço final ao salvar.

#### Scenario: Edição com dados válidos
- **WHEN** o usuário edita um produto existente e submete dados válidos incluindo `unit` e `final_price >= CMV`
- **THEN** o sistema atualiza o registro em `Products` com os novos valores, recalcula `markup` e `profit_margin` derivados do `final_price`, e atualiza `updated_at`

#### Scenario: Preço abaixo do custo na edição
- **WHEN** o usuário submete `final_price < CMV`
- **THEN** o sistema retorna erro de prejuízo sem alterar o registro

#### Scenario: Produto inexistente
- **WHEN** o comando `update_product` é invocado com `id` que não existe
- **THEN** o sistema retorna erro sem alterar nenhum registro

### Requirement: Precificação automática no formulário

O sistema SHALL permitir que o operador informe o **preço final** e SHALL calcular e exibir em tempo real o CMV e a **Margem de lucro derivada** conforme o usuário edita `cost_price`, `expected_loss_rate` e `final_price`, usando as taxas de `get_pricing_rates` (cujo `cv` já inclui a taxa de cartão embutida). O sistema NÃO SHALL exibir a porcentagem de markup no formulário (o `markup` continua sendo derivado e persistido, apenas não é mostrado). O preço de venda não SHALL ser recalculado automaticamente a partir do histórico da empresa.

#### Scenario: Cálculo com histórico de vendas disponível
- **WHEN** o formulário é aberto para um novo produto e existem vendas nos últimos 12 meses
- **THEN** CV% é derivado de (despesas variáveis de `Financial_Entries` + taxas de cartão pagas de `Sale_Payments`) ÷ faturamento dos últimos 12 meses; DF% é derivado das despesas fixas ÷ faturamento; o campo `final_price` é pré-populado com um **preço sugerido** derivado da margem realizada (`get_realized_margin`)

#### Scenario: Cálculo sem histórico de vendas
- **WHEN** o formulário é aberto e não existem vendas nos últimos 12 meses
- **THEN** CV% = 0, DF% = 0 e o preço sugerido recai sobre o preço de equilíbrio (margem zero); o formulário exibe aviso de que as taxas não puderam ser calculadas

#### Scenario: Atualização em tempo real
- **WHEN** o usuário altera `cost_price`, `expected_loss_rate` ou `final_price`
- **THEN** o CMV e a Margem de lucro derivada são recalculados instantaneamente no cliente sem nova requisição, mantendo o `final_price` informado pelo operador como entrada estável

#### Scenario: Markup não exibido no formulário
- **WHEN** o formulário de produto é renderizado em qualquer estado
- **THEN** a porcentagem de markup não é exibida; apenas o custo, o CMV e a margem derivada aparecem como leitura

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

## REMOVED Requirements

### Requirement: Detecção e aviso de margem inviável

**Reason**: A precificação deixa de ser dirigida pela margem; a inviabilidade e o piso passam a ser expressos no espaço de preço (piso de CMV, preço de equilíbrio e taxas que consomem o preço), coberta pela nova requirement "Piso de custo e margem derivada do preço final".
**Migration**: Consumidores que liam `feasible`/`max_margin` de `computePrice` no formulário passam a usar `computeMarginFromPrice`, que retorna `feasible`, `below_cost`, `min_price` e `break_even_price`. `computePrice` permanece disponível para gerar o preço sugerido inicial.

### Requirement: Validação dinâmica do campo de margem

**Reason**: O campo de margem deixa de ser editável (a margem é derivada do preço), substituído pela validação dinâmica do campo de preço final.
**Migration**: A validação de limite migra do campo de margem para o campo de preço final, com mínimo dinâmico igual ao CMV; ver "Validação dinâmica do campo de preço final".

## ADDED Requirements

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

O sistema SHALL restringir dinamicamente o valor mínimo aceito no campo de preço final do formulário de produto ao CMV atual (`cost_price / (1 − expected_loss_rate)`), recalculado conforme `cost_price` e `expected_loss_rate` mudam.

#### Scenario: Mínimo do campo derivado do CMV atual
- **WHEN** o formulário de produto é renderizado com `cost_price` e `expected_loss_rate`
- **THEN** o atributo de valor mínimo do campo de preço final corresponde ao CMV calculado

#### Scenario: Mínimo acompanha mudança de custo ou perda
- **WHEN** o usuário altera `cost_price` ou `expected_loss_rate`
- **THEN** o valor mínimo do campo de preço final é recalculado para refletir o novo CMV
