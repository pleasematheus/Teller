## Purpose

Controle de estoque de insumos da loja de fotos: consulta de saldo atual, registro de compras multi-item com movimentações automáticas, lançamento manual de perdas e ajustes, histórico de movimentações por produto e indicador de capacidade produtiva estimada.

## Requirements

### Requirement: Consulta de saldo de estoque

O sistema SHALL retornar o saldo atual de todos os insumos a partir da view `Stock`, incluindo produtos sem movimentações (saldo zero).

#### Scenario: Saldo com movimentações
- **WHEN** o comando `list_stock` é invocado
- **THEN** o sistema retorna `product_id`, `name`, `unit` e `current_quantity` para cada produto com movimentações, com `current_quantity = Σin - Σout`

#### Scenario: Produto sem movimentações
- **WHEN** um produto não possui registros em `Stock_Movements`
- **THEN** o sistema retorna `current_quantity = 0` para esse produto

### Requirement: Busca e filtro por tipo na consulta de saldo de estoque

A tela de saldo de estoque SHALL oferecer um campo de busca por nome e um filtro por tipo (Todos/Vendáveis/Insumos) sobre os itens retornados por `list_stock`. A busca SHALL ignorar acentuação e caixa e ranquear correspondências por prefixo, operando no cliente sem nova requisição. Como `StockItem` não traz o atributo `sellable`, o tipo de cada item SHALL ser derivado do cruzamento de `product_id` com os produtos de `list_products` (já carregados pela tela). O filtro SHALL recortar o conjunto e a busca SHALL ranquear o subconjunto.

#### Scenario: Busca por nome insensível a acento
- **WHEN** o usuário digita "acai" e existe um item de estoque "Açaí 300ml"
- **THEN** a tabela exibe a linha de "Açaí 300ml"

#### Scenario: Filtro "Vendáveis" no estoque
- **WHEN** o usuário seleciona o filtro "Vendáveis"
- **THEN** a tabela exibe apenas itens cujo `product_id` corresponde a um produto com `sellable = true`

#### Scenario: Filtro "Insumos" no estoque
- **WHEN** o usuário seleciona o filtro "Insumos"
- **THEN** a tabela exibe apenas itens cujo `product_id` corresponde a um produto com `sellable = false`

#### Scenario: Busca combinada com filtro de tipo
- **WHEN** o filtro "Insumos" está ativo e o usuário digita um termo de busca
- **THEN** a tabela exibe apenas itens de insumo cujo nome casa com o termo

#### Scenario: Resultado vazio distinto de estoque sem cadastro
- **WHEN** existem itens em estoque mas nenhum casa com a busca ou o filtro ativo
- **THEN** a tela exibe uma mensagem de "nenhum resultado", distinta da mensagem de estoque sem produtos cadastrados

#### Scenario: Busca vazia preserva o saldo filtrado
- **WHEN** o campo de busca está vazio
- **THEN** a tabela exibe todos os itens do filtro de tipo ativo, sem reordenação por busca

### Requirement: Ordenação por coluna na consulta de saldo de estoque

A tela de saldo de estoque SHALL permitir ordenar os itens por qualquer coluna de dados: Produto (nome), Unidade, Quantidade (`current_quantity`) e Capacidade estimada. Cada cabeçalho de coluna de dados SHALL ser clicável e alternar ciclicamente entre três estados: ascendente, descendente e sem ordenação (ordem padrão). No máximo uma coluna SHALL estar ativa por vez; clicar numa coluna diferente da ativa SHALL iniciar a ordenação em ascendente naquela coluna. O estado de ordenação ativo SHALL ser indicado visualmente no cabeçalho da coluna ativa.

A ordenação SHALL operar no cliente, sem nova requisição, sobre o conjunto já recortado pelo filtro de tipo e pela busca por nome. Quando uma ordenação está ativa, ela SHALL ter precedência sobre o ranking por relevância de nome da busca; quando nenhuma está ativa, a ordem é a do ranking de busca (com busca preenchida) ou a ordem padrão (com busca vazia).

Critérios por coluna:
- Produto e Unidade SHALL ser ordenadas como texto, insensível a caixa, respeitando a colação do português (acentuação).
- Quantidade SHALL ser ordenada numericamente por `current_quantity`.
- Capacidade estimada SHALL ser ordenada numericamente pela MAIOR capacidade estimada da linha; linhas sem capacidade ("—") SHALL aparecer sempre por último, independentemente da direção.

#### Scenario: Ordenar coluna ascendente
- **WHEN** o usuário clica no cabeçalho de uma coluna de dados sem ordenação ativa nela
- **THEN** a tabela exibe os itens ordenados por aquela coluna em ordem ascendente, e o cabeçalho indica a direção ascendente

#### Scenario: Alternar para descendente
- **WHEN** o usuário clica novamente no cabeçalho da coluna já ordenada em ascendente
- **THEN** a tabela passa a exibir os itens em ordem descendente naquela coluna, e o cabeçalho indica a direção descendente

#### Scenario: Remover ordenação
- **WHEN** o usuário clica no cabeçalho da coluna já ordenada em descendente
- **THEN** a ordenação é removida e a tabela volta à ordem padrão (ou ao ranking de busca, se houver termo de busca)

#### Scenario: Apenas uma coluna ativa
- **WHEN** uma coluna está ordenada e o usuário clica no cabeçalho de outra coluna
- **THEN** a ordenação anterior é substituída e a nova coluna passa a ordenar em ascendente

#### Scenario: Ordenação textual de Produto e Unidade
- **WHEN** o usuário ordena pela coluna Produto ou Unidade
- **THEN** os itens são ordenados alfabeticamente respeitando acentuação e ignorando caixa

#### Scenario: Capacidade pela maior estimativa
- **WHEN** o usuário ordena pela coluna Capacidade estimada em descendente
- **THEN** os itens são ordenados pela maior capacidade estimada da linha, do maior para o menor

#### Scenario: Linhas sem capacidade vão ao fim
- **WHEN** o usuário ordena pela coluna Capacidade estimada em qualquer direção
- **THEN** as linhas sem capacidade ("—") aparecem por último

#### Scenario: Ordenação opera sobre o conjunto filtrado e buscado
- **WHEN** um filtro de tipo e/ou um termo de busca estão ativos e o usuário ativa uma ordenação
- **THEN** apenas os itens que passam pelo filtro e pela busca são exibidos, ordenados pela coluna escolhida

#### Scenario: Ordenação tem precedência sobre o ranking de busca
- **WHEN** há um termo de busca preenchido e uma ordenação de coluna ativa
- **THEN** os resultados são ordenados pela coluna escolhida, ignorando o ranking por prefixo de nome

### Requirement: Indicador de capacidade produtiva

O sistema SHALL calcular e exibir, para cada insumo, a quantidade estimada de produtos finais que podem ser produzidos com o saldo atual, com base na composição definida em `Product_Components`.

#### Scenario: Insumo com componentes cadastrados
- **WHEN** o módulo de estoque é aberto e o insumo faz parte de ao menos um produto via `Product_Components`
- **THEN** o sistema exibe "≈ N [produto]s possíveis" calculado como `current_quantity ÷ calculated_quantity`

#### Scenario: Insumo sem componentes cadastrados
- **WHEN** o insumo não participa de nenhum `Product_Components`
- **THEN** o indicador de capacidade não é exibido para esse insumo

### Requirement: Registro de compra multi-item

O sistema SHALL registrar uma compra com múltiplos itens como **rascunho** em uma única operação, criando o registro em `Purchases` com `status = 'open'` e inserindo cada item em `Purchase_Items`, **sem** criar movimentações de estoque e **sem** alterar custo, preço ou taxa de perda dos produtos. A aplicação desses efeitos ocorre apenas no fechamento (ver "Fechamento de compra"). Cada item SHALL referenciar um produto **existente** (via `product_id`) **ou** ser um **produto novo**, caso em que `product_id` é nulo e o item guarda os campos pendentes `new_product_name`, `new_product_unit`, `new_product_description` (opcional) e `new_product_sellable`; exatamente uma das duas formas SHALL estar presente por item, e nenhum produto SHALL ser criado em `Products` enquanto a compra está em rascunho. Cada item SHALL guardar `quantity`, `unit_cost`, o fator de conversão opcional `unit_qty` e o `final_price` pretendido. A conversão de unidade de compra para unidade de estoque SHALL usar o fator único `unit_qty`: a quantidade que entrará em estoque ao fechar é `quantity * unit_qty` quando `unit_qty` está preenchido, ou `quantity` quando `unit_qty` é nulo. A seleção de produto de cada linha SHALL usar o combobox de seleção de produto (ver capability `product-search`), que oferece a ação de criar produto novo. Cada linha SHALL expor um campo de preço de venda editável (`final_price`) pré-preenchido pelo preço sugerido (`computePrice`); o operador SHALL poder sobrescrevê-lo. Edição de itens e do frete SHALL ser permitida apenas enquanto `status = 'open'`.

#### Scenario: Criação de rascunho de compra
- **WHEN** o comando `create_purchase` é invocado com `freight >= 0` e ao menos um item válido
- **THEN** o sistema insere em `Purchases` com `status = 'open'` e insere cada item em `Purchase_Items`, sem criar `Stock_Movements` e sem alterar `cost_price`, `final_price`, `markup`, `profit_margin`, `expected_loss_rate` ou `loss_mode` de nenhum produto

#### Scenario: Rascunho com item de produto novo
- **WHEN** um item do rascunho não tem `product_id` e traz `new_product_name`, `new_product_unit`, `new_product_description` (opcional) e `new_product_sellable`
- **THEN** o sistema persiste esses campos pendentes em `Purchase_Items` com `product_id` nulo e NÃO cria registro em `Products`

#### Scenario: Item sem produto existente nem dados de produto novo
- **WHEN** um item não tem `product_id` e também não tem `new_product_name`/`new_product_unit`
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

#### Scenario: Preço de venda sugerido por item
- **WHEN** o operador define produto, quantidade e custo de uma linha (e o frete da compra)
- **THEN** o sistema exibe no campo de preço de venda da linha o preço sugerido por `computePrice`, calculado sobre o custo efetivo do lote (incluindo a parcela de frete rateada), a taxa de perda do insumo e as taxas globais

#### Scenario: Operador sobrescreve o preço de venda
- **WHEN** o operador edita o campo de preço de venda de uma linha
- **THEN** o valor informado passa a ser o `final_price` enviado no payload de `create_purchase` para aquele item, ignorando o preço sugerido

#### Scenario: Seleção de produto via combobox
- **WHEN** o operador abre o seletor de produto de uma linha de item e escolhe um produto no combobox
- **THEN** a linha passa a exibir o nome do produto escolhido, o `productId` da linha é definido e o sistema dispara `get_product_loss_rate` para preencher a perda sugerida daquela linha

#### Scenario: Conversão por fator único
- **WHEN** um item de compra tem `unit_qty` preenchido
- **THEN** a quantidade considerada para estoque no fechamento é `quantity * unit_qty`

#### Scenario: Sem conversão de unidade
- **WHEN** um item de compra tem `unit_qty` nulo
- **THEN** a quantidade considerada para estoque no fechamento é igual à `quantity` do item

#### Scenario: Edição bloqueada após fechamento
- **WHEN** uma compra tem `status = 'closed'` e o operador tenta alterar seus itens ou frete
- **THEN** o sistema rejeita a alteração

#### Scenario: Item com quantidade zero ou negativa
- **WHEN** qualquer item da compra tem `quantity <= 0`
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

### Requirement: Fechamento de compra

O sistema SHALL fornecer o comando `close_purchase(purchase_id)` que, em uma única transação, aplica os efeitos de uma compra em rascunho e marca `status = 'closed'`. Antes de aplicar os efeitos de cada item, se o item for de **produto novo** (`product_id` nulo com campos `new_product_*`), o sistema SHALL criar o produto em `Products` (validando nome e unidade ativa em `Units_Of_Measure`, com `cost_price=0`, `markup=0`, `final_price=0`, `profit_margin=0`, `expected_loss_rate=0` iniciais), fazer backfill de `Purchase_Items.product_id` com o id gerado, e então tratá-lo como um item de produto existente. Para cada item com `creates_stock = true`, o sistema SHALL: fixar a taxa de perda do insumo (modo `auto`/`manual` com override por item, gravando em `Purchase_Items.loss_rate` e atualizando `Products.expected_loss_rate` e `Products.loss_mode`); recalcular `Products.cost_price` como média ponderada por quantidade incluindo o frete rateado proporcional ao valor da linha; persistir `Products.final_price` e derivar `markup` e `profit_margin` como snapshot (`cmv = cost_price/(1 − expected_loss_rate)`, `markup = final_price/cmv`, `profit_margin = 1 − cv − df − investment_rate − cmv/final_price`); e criar `Stock_Movements (movement_type='in', motive='compra')` com quantidade `quantity * unit_qty` (ou `quantity` quando `unit_qty` é nulo). O fechamento SHALL ser idempotente em relação ao estado: uma compra já `closed` não SHALL ser fechada novamente.

#### Scenario: Fechamento aplica estoque e custo
- **WHEN** `close_purchase` é invocado para uma compra `open` com itens válidos
- **THEN** o sistema cria `Stock_Movements ('in','compra')` para cada item, atualiza `expected_loss_rate`, `loss_mode`, `cost_price` (média ponderada), `final_price`, `markup` e `profit_margin` dos produtos, e marca `Purchases.status = 'closed'` — tudo em uma transação

#### Scenario: Fechamento cria o produto novo
- **WHEN** `close_purchase` é invocado para um rascunho que contém um item de produto novo válido
- **THEN** o sistema cria o produto em `Products`, faz backfill de `product_id` no item, e aplica custo/perda/preço/movimentação de estoque para esse produto, tudo na mesma transação

#### Scenario: Produto novo com unidade inválida
- **WHEN** um item de produto novo tem `new_product_unit` inexistente ou inativa em `Units_Of_Measure`
- **THEN** o fechamento retorna erro, a transação sofre rollback e o `status` permanece `open`

#### Scenario: Produto novo não persiste quando o fechamento falha
- **WHEN** o fechamento de um rascunho com item de produto novo falha em um passo posterior
- **THEN** a transação sofre rollback e nenhum produto permanece em `Products`

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

### Requirement: Lançamento manual de perda ou ajuste

O sistema SHALL permitir registrar uma movimentação manual de estoque para cobrir perdas (impressão errada) e correções de inventário.

#### Scenario: Registro de perda
- **WHEN** o comando `create_stock_adjustment` é invocado com `motive = 'perda'` e `quantity > 0`
- **THEN** o sistema cria `Stock_Movement (movement_type='out', motive='perda')` com a quantidade informada

#### Scenario: Ajuste positivo (entrada)
- **WHEN** o comando `create_stock_adjustment` é invocado com `motive = 'ajuste'` e `quantity > 0`
- **THEN** o sistema cria `Stock_Movement (movement_type='in', motive='ajuste')`

#### Scenario: Ajuste negativo (saída)
- **WHEN** o comando `create_stock_adjustment` é invocado com `motive = 'ajuste'` e `quantity < 0`
- **THEN** o sistema cria `Stock_Movement (movement_type='out', motive='ajuste')` com `abs(quantity)`

#### Scenario: Quantidade zero
- **WHEN** `quantity = 0` é informado
- **THEN** o sistema retorna erro de validação sem criar movimentação

### Requirement: Histórico de movimentações por produto

O sistema SHALL retornar o histórico completo de movimentações de um produto específico, ordenado do mais recente para o mais antigo. Cada movimentação SHALL incluir o `sale_id` da venda que a originou quando a movimentação for de venda, e `null` para movimentações sem venda associada (compra, perda, ajuste).

#### Scenario: Produto com histórico
- **WHEN** o comando `list_stock_movements` é invocado com `product_id` válido
- **THEN** o sistema retorna todos os registros de `Stock_Movements` daquele produto com `id`, `quantity`, `movement_type`, `motive`, `created_at` e `sale_id`, ordenados por `created_at DESC`

#### Scenario: Produto sem histórico
- **WHEN** o produto não possui movimentações
- **THEN** o sistema retorna lista vazia sem erro

#### Scenario: Movimentação de venda traz o ID da venda
- **WHEN** uma movimentação foi criada pelo fechamento de uma venda (`movement_type='out'`, `motive='venda'`)
- **THEN** o registro retornado tem `sale_id` igual ao `id` da venda que a originou

#### Scenario: Movimentação sem venda associada
- **WHEN** uma movimentação tem `motive` em (`compra`, `perda`, `ajuste`)
- **THEN** o registro retornado tem `sale_id = null`

#### Scenario: Exibição do ID da venda na tela de estoque
- **WHEN** o histórico de movimentações é exibido e uma movimentação possui `sale_id`
- **THEN** a tela exibe o identificador da venda para essa linha; movimentações com `sale_id = null` exibem o campo vazio

### Requirement: Listagem de componentes de produto

O sistema SHALL retornar os componentes (insumos) de um produto específico com suas quantidades, para uso no cálculo do indicador de capacidade.

#### Scenario: Produto com componentes
- **WHEN** o comando `list_product_components` é invocado com `product_id` válido
- **THEN** o sistema retorna `child_id`, `child_name`, `quantity`, `calculated_quantity` e `use_calculated` para cada componente

#### Scenario: Produto sem componentes
- **WHEN** o produto não possui entradas em `Product_Components`
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Baixa de estoque no fechamento da venda registra a venda de origem

O sistema SHALL registrar, em cada movimentação de saída criada no fechamento de uma venda, o `sale_id` da venda que a originou, tanto para produtos sem componentes quanto para insumos consumidos via `Product_Components`.

#### Scenario: Baixa de produto sem componentes
- **WHEN** uma venda é fechada e um produto vendido não possui componentes em `Product_Components`
- **THEN** a `Stock_Movement (movement_type='out', motive='venda')` criada para esse produto tem `sale_id` igual ao `id` da venda

#### Scenario: Baixa de insumos via BOM
- **WHEN** uma venda é fechada e um produto vendido possui componentes em `Product_Components`
- **THEN** cada `Stock_Movement (movement_type='out', motive='venda')` criada para os insumos consumidos tem `sale_id` igual ao `id` da venda

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

### Requirement: Botão de ajuste visualmente clicável

O botão "Ajuste" na tabela de saldo de estoque SHALL usar `variant="secondary"` para ter aparência de ação clicável, distinguindo-o visualmente das células da tabela.

#### Scenario: Botão Ajuste renderizado na tabela de saldo

- **WHEN** a tabela de saldo de estoque é exibida com ao menos um produto
- **THEN** cada linha exibe o botão "Ajuste" com fundo sólido (`variant="secondary"`), não transparente

### Requirement: Custo médio ponderado do insumo na compra

O sistema SHALL recalcular o `Products.cost_price` de cada insumo comprado como **média ponderada por quantidade em estoque**, tornando a compra a única fonte do custo do insumo (espelhando o que já ocorre com `expected_loss_rate`). O custo total do lote de um item SHALL ser `quantity × unit_cost` somado à **parcela de frete rateada** para aquele item. O frete SHALL ser rateado entre os itens proporcionalmente ao valor de cada linha: `freight_share_i = freight × (quantity_i × unit_cost_i) ÷ Σ(quantity × unit_cost)`; quando o somatório dos valores de linha é zero, nenhuma parcela de frete é rateada. O custo unitário do lote SHALL ser o custo total do lote (mercadoria + frete rateado) dividido pela quantidade efetivamente movimentada em estoque (após conversão caixa→unidade). O novo custo SHALL ser `(saldo_atual × cost_price_atual + qtd_estoque_lote × custo_unitário_lote) / (saldo_atual + qtd_estoque_lote)`, usando o saldo de estoque anterior à entrada do lote. O `freight` SHALL continuar gravado em `Purchases.total_cost`.

#### Scenario: Primeira compra de um insumo sem estoque
- **WHEN** uma compra fixa o custo de um insumo cujo saldo de estoque anterior é 0
- **THEN** `cost_price` passa a ser o custo unitário de estoque do lote (`(quantity × unit_cost + frete_rateado) ÷ qtd_estoque_lote`), sem diluição por saldo anterior

#### Scenario: Compra subsequente com estoque existente
- **WHEN** uma compra fixa o custo de um insumo que já tem saldo de estoque com um `cost_price` vigente
- **THEN** `cost_price` passa a ser a média ponderada `(saldo_atual × cost_price_atual + qtd_estoque_lote × custo_unitário_lote) / (saldo_atual + qtd_estoque_lote)`

#### Scenario: Frete rateado entra no custo do insumo
- **WHEN** uma compra com `freight > 0` e mais de um item é registrada
- **THEN** cada insumo recebe uma parcela de frete proporcional ao valor da sua linha (`freight × valor_linha ÷ subtotal`), somada ao custo do lote antes do cálculo da média ponderada, e o `freight` total permanece gravado em `Purchases.total_cost`

#### Scenario: Frete não rateado quando o subtotal é zero
- **WHEN** uma compra com `freight > 0` é registrada e o somatório dos valores de linha é zero
- **THEN** nenhuma parcela de frete é somada ao `cost_price` de qualquer insumo e o `freight` permanece apenas em `Purchases.total_cost`

#### Scenario: Conversão por fator único no custo unitário
- **WHEN** um item de compra tem `unit_qty` preenchido (a quantidade de estoque difere da quantidade comprada)
- **THEN** o custo unitário de estoque do lote é `(quantity × unit_cost + frete_rateado) ÷ (quantity × unit_qty)`, mantendo a média ponderada coerente com a unidade de estoque

#### Scenario: Custo congelado entre compras
- **WHEN** movimentos de venda ou perda ocorrem após uma compra, sem nova compra do insumo
- **THEN** `cost_price` permanece com o valor fixado na última compra até a próxima compra do insumo o refixar

### Requirement: Quantidade de item de compra respeita o flag inteiro da unidade

O campo de quantidade de cada linha de item de compra SHALL respeitar o flag `integer` da unidade do produto da linha. Quando a unidade do produto tem `integer = true`, o formulário SHALL usar passo inteiro (`step="1"`), SHALL normalizar a quantidade para o inteiro mais próximo (mínimo 1) quando o campo perde o foco, e SHALL bloquear o envio de uma `quantity` com casas decimais como defesa secundária. O campo "Unidades por caixa" (`unit_qty`) também SHALL usar passo inteiro e bloquear decimais quando a unidade do produto for inteira; quando a conversão estiver ativa e a unidade do produto for inteira, a quantidade de estoque resultante (`quantity * unit_qty`) SHALL também ser inteira, caso contrário o submit SHALL ser bloqueado. Quando a unidade tem `integer = false`, casas decimais SHALL ser permitidas tanto em `quantity` quanto em `unit_qty`. Quando a unidade do produto não for encontrada na lista de unidades, o sistema SHALL tratá-la como inteira (bloqueia decimais), coerente com o default.

#### Scenario: Digitação fracionada em unidade inteira é normalizada
- **WHEN** o operador digita uma quantidade com casas decimais em um item cuja unidade é inteira e o campo perde o foco
- **THEN** o valor é normalizado para o inteiro mais próximo, com mínimo de 1
- **THEN** enquanto o valor permanecer fracionado, o campo exibe o aviso "Apenas inteiros"

#### Scenario: Unidade inteira bloqueia decimal
- **WHEN** o produto da linha usa uma unidade com `integer = true` e o operador informa `quantity = 2,5`
- **THEN** o formulário exibe erro de validação e não envia o item

#### Scenario: Unidade inteira aceita inteiro
- **WHEN** o produto da linha usa uma unidade com `integer = true` e o operador informa `quantity = 3`
- **THEN** o item é aceito com `quantity = 3`

#### Scenario: Unidade decimal aceita decimal
- **WHEN** o produto da linha usa uma unidade com `integer = false` e o operador informa `quantity = 2,5`
- **THEN** o item é aceito com `quantity = 2,5`

#### Scenario: Unidade ausente assume inteiro
- **WHEN** a unidade do produto da linha não consta na lista de unidades carregada
- **THEN** o campo de quantidade usa passo inteiro e bloqueia o envio de quantidade decimal

#### Scenario: Conversão caixa para unidade gera estoque fracionado em unidade inteira
- **WHEN** um item tem unidade de produto inteira, a conversão `unit_qty` ativa, e `quantity * unit_qty` resulta em valor fracionado
- **THEN** o sistema bloqueia o submit e sinaliza que a quantidade de estoque resultante deve ser inteira
