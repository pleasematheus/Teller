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

### Requirement: Cálculo automático de taxa de perda

O sistema SHALL calcular `expected_loss_rate` a partir do histórico de `Stock_Movements` do produto quando o usuário solicitar, como alternativa ao preenchimento manual. O cálculo SHALL ocorrer apenas mediante ação explícita do usuário ("Calcular"); o sistema NÃO recalcula `expected_loss_rate` automaticamente na leitura ou exibição, mesmo quando `loss_mode = auto`.

#### Scenario: Cálculo com histórico disponível
- **WHEN** o usuário aciona "Calcular" para perda automática em um produto existente
- **THEN** o sistema retorna `SUM(quantity WHERE movement_type='out' AND motive='perda') / SUM(quantity WHERE movement_type='in')` para aquele produto

#### Scenario: Cálculo sem histórico de movimentos
- **WHEN** o produto não possui `Stock_Movements`
- **THEN** o sistema retorna 0 sem erro

#### Scenario: Modo auto indisponível em criação
- **WHEN** o formulário é aberto para criação de novo produto
- **THEN** a opção de cálculo automático de perda está desabilitada (produto ainda não tem histórico)

#### Scenario: Valor congelado entre cálculos
- **WHEN** um produto está salvo com `loss_mode = auto` e novos movimentos de perda são registrados no estoque
- **THEN** `expected_loss_rate` permanece com o último valor salvo até o usuário acionar "Calcular" e salvar novamente

### Requirement: Persistência do modo da taxa de perda

O sistema SHALL persistir o modo da taxa de perda (`loss_mode`, com valores `manual` ou `auto`) por produto, com default `manual`, e restaurar o modo selecionado ao reabrir o produto para edição. O modo é uma preferência salva: alterar o modo não recalcula nem altera `expected_loss_rate`; o valor só muda por entrada manual ou ao acionar "Calcular". Tanto o modo quanto o valor permanecem inalterados até a próxima edição salva.

#### Scenario: Persistir modo ao salvar
- **WHEN** o usuário salva um produto com `loss_mode = auto`
- **THEN** o sistema grava `loss_mode = auto` no registro do produto

#### Scenario: Restaurar modo na edição
- **WHEN** o usuário reabre para edição um produto salvo com `loss_mode = auto`
- **THEN** o formulário exibe o modo `auto` selecionado, sem reiniciar para `manual`

#### Scenario: Default manual
- **WHEN** um produto é criado sem `loss_mode` informado ou é lido de um registro legado sem o campo
- **THEN** o sistema assume `loss_mode = manual`

#### Scenario: Modo inválido rejeitado
- **WHEN** o usuário submete um `loss_mode` diferente de `manual` ou `auto`
- **THEN** o sistema rejeita com erro de validação

#### Scenario: Alterar modo não altera o valor
- **WHEN** o usuário troca `loss_mode` entre `manual` e `auto` sem acionar "Calcular" nem digitar
- **THEN** `expected_loss_rate` permanece com o valor atual, sem recálculo

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

O sistema SHALL restringir dinamicamente o valor mínimo aceito no campo de preço final do formulário de produto ao CMV atual (`cost_price / (1 − expected_loss_rate)`), recalculado conforme `cost_price` e `expected_loss_rate` mudam.

#### Scenario: Mínimo do campo derivado do CMV atual
- **WHEN** o formulário de produto é renderizado com `cost_price` e `expected_loss_rate`
- **THEN** o atributo de valor mínimo do campo de preço final corresponde ao CMV calculado

#### Scenario: Mínimo acompanha mudança de custo ou perda
- **WHEN** o usuário altera `cost_price` ou `expected_loss_rate`
- **THEN** o valor mínimo do campo de preço final é recalculado para refletir o novo CMV

### Requirement: Cálculo de margem realizada

O sistema SHALL calcular a margem de lucro média realizada dos últimos 12 meses a partir das vendas fechadas.

#### Scenario: Cálculo com vendas disponíveis
- **WHEN** o command `get_realized_margin` é invocado
- **THEN** o sistema retorna a média de `(sp.discounted_price - cmv × sp.quantity) / sp.discounted_price` para todos os itens de `Sales_Products` onde `Sales.status = 'closed'` e `Sales.created_at >= NOW() - INTERVAL 12 MONTHS`, onde `cmv = cost_price / (1 - expected_loss_rate)`

#### Scenario: Sem vendas nos últimos 12 meses
- **WHEN** não há vendas fechadas nos últimos 12 meses
- **THEN** o command retorna `0` sem erro

