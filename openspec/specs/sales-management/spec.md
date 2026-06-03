## Purpose

Registro e gerenciamento de pedidos de venda da loja de fotos. Fluxo: pedido aberto (itens a preço cheio) → fechamento com desconto e pagamento → disparo de produção. Clientes identificados por nome livre; pagamento suporta múltiplos métodos.

## Requirements

### Requirement: Listagem de pedidos em aberto

O sistema SHALL retornar todos os pedidos com `status = 'open'`, ordenados do mais recente para o mais antigo, com campos suficientes para exibição na lista.

#### Scenario: Pedidos em aberto existentes
- **WHEN** o command `list_sales` é invocado com `status = 'open'`
- **THEN** o sistema retorna `id`, `label`, `total_value`, `created_at` de cada pedido com `status = 'open'`, ordenados por `created_at DESC`

#### Scenario: Nenhum pedido em aberto
- **WHEN** não há pedidos com `status = 'open'`
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Histórico de pedidos fechados

O sistema SHALL retornar pedidos com `status = 'closed'` para consulta, sem permitir edição.

#### Scenario: Histórico com vendas fechadas
- **WHEN** o command `list_sales` é invocado com `status = 'closed'`
- **THEN** o sistema retorna `id`, `label`, `total_value`, `created_at` ordenados por `created_at DESC`

#### Scenario: Histórico vazio
- **WHEN** não há pedidos fechados
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Criação de pedido

O sistema SHALL permitir criar um pedido com identificação livre do cliente e ao menos um item. O pedido é criado a preço cheio, sem desconto; o desconto é concedido apenas no fechamento. A adição de itens SHALL usar o combobox de seleção de produto (ver capability `product-search`), com um campo de quantidade ao lado da busca.

#### Scenario: Criação com dados válidos
- **WHEN** o command `create_sale` é invocado com `user_id` válido, ao menos um item com `product_id` válido e `quantity >= 1`
- **THEN** o sistema insere o pedido em `Sales (status='open')`, insere cada item em `Sales_Products` com `price = Products.final_price`, `discount = 0` e `discounted_price = price`, calcula `total_value = SUM(price × quantity)` e retorna o `id` do pedido criado

#### Scenario: Formulário de pedido sem desconto
- **WHEN** o operador monta um pedido no formulário de criação
- **THEN** o formulário não exibe campo de desconto por item, desconto global nem indicador de desconto máximo, e o total exibido é `SUM(final_price × quantity)`

#### Scenario: Adição de produto com quantidade
- **WHEN** o operador informa uma quantidade no campo ao lado da busca e seleciona um produto no combobox
- **THEN** o item é adicionado ao carrinho com a quantidade informada (default 1 quando não informada) e o campo de quantidade volta a 1

#### Scenario: Seleção de produto já no carrinho
- **WHEN** o operador seleciona um produto que já está no carrinho
- **THEN** a quantidade existente daquele item é incrementada pela quantidade informada em vez de criar uma linha duplicada

#### Scenario: Produto no carrinho marcado na lista
- **WHEN** o combobox está aberto e um produto da lista já consta no carrinho
- **THEN** esse produto é exibido com uma marcação indicando que já está no pedido

#### Scenario: Pedido sem itens
- **WHEN** o command `create_sale` é invocado sem itens
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

#### Scenario: Quantidade inválida em item
- **WHEN** qualquer item possui `quantity <= 0`
- **THEN** o sistema retorna erro de validação sem criar nenhum registro

#### Scenario: label vazio
- **WHEN** o command `create_sale` é invocado com `label` em branco
- **THEN** o sistema aceita e cria o pedido com `label = ''`

### Requirement: Edição de pedido em aberto

O sistema SHALL permitir atualizar os itens e o `label` de um pedido com `status = 'open'`. A edição também ocorre a preço cheio, sem desconto.

#### Scenario: Edição com dados válidos
- **WHEN** o command `update_sale` é invocado com `id` de pedido `open` e lista de itens válida
- **THEN** o sistema substitui os itens em `Sales_Products` com `discount = 0` e `discounted_price = price` e recalcula `total_value = SUM(price × quantity)`

#### Scenario: Edição de pedido fechado
- **WHEN** o command `update_sale` é invocado com `id` de pedido `closed`
- **THEN** o sistema retorna erro sem alterar nenhum registro

#### Scenario: Pedido inexistente
- **WHEN** o command `update_sale` é invocado com `id` que não existe
- **THEN** o sistema retorna erro sem alterar nenhum registro

### Requirement: Indicador de desconto máximo por item

O sistema SHALL calcular, na tela de fechamento, para cada item do pedido, o percentual máximo de desconto aplicável sem que o preço descontado fique abaixo do piso de margem. Como a taxa de cartão já está embutida no `cv`, o piso não soma mais um termo de taxa separado.

#### Scenario: Cálculo com todos os dados disponíveis
- **WHEN** a tela de fechamento exibe um item com `cost_price`, `expected_loss_rate`, `cv` e `df` disponíveis
- **THEN** o frontend calcula `floor = cost_price / (1 - expected_loss_rate) + (cv + df) × final_price` e exibe `max_discount_pct = (final_price - floor) / final_price` e `max_discount_value = final_price - floor` de forma discreta

#### Scenario: Indicador visível apenas quando relevante
- **WHEN** o campo de desconto do item não está focado E `discount = 0`
- **THEN** o indicador de desconto máximo não é exibido

#### Scenario: Indicador visível ao interagir
- **WHEN** o campo de desconto do item está focado OU `discount > 0`
- **THEN** o indicador exibe `· máx. R$ X (Y%)`

#### Scenario: Desconto acima do piso
- **WHEN** o desconto aplicado resulta em `discounted_price < floor`
- **THEN** o campo de desconto exibe aviso visual (ex: cor vermelha ou ícone) indicando que o item está abaixo do piso de margem

### Requirement: Modalidade de desconto por item no fechamento

O sistema SHALL permitir, na tela de fechamento, escolher para cada item a modalidade do desconto: porcentagem (%) ou valor absoluto (R$). A modalidade é estado de interface; o valor enviado ao backend SHALL ser sempre a fração de desconto (0-1) por item, independentemente da modalidade escolhida. A modalidade inicial de cada item SHALL ser porcentagem (%).

#### Scenario: Modalidade inicial em porcentagem
- **WHEN** a tela de fechamento carrega os itens do pedido
- **THEN** cada item exibe o campo de desconto em modo `%`, com o comportamento atual preservado (entrada em porcentagem convertida para fração `valor / 100`)

#### Scenario: Desconto por item em valor (R$)
- **WHEN** o operador alterna a modalidade de um item para `R$` e informa um valor absoluto `v`
- **THEN** o frontend calcula a fração `discount = clamp(v / (price × quantity), 0, 1)` para aquele item e usa essa fração no envio ao `close_sale`

#### Scenario: Desconto por item em porcentagem (%)
- **WHEN** o operador mantém ou alterna a modalidade de um item para `%` e informa um percentual `p`
- **THEN** o frontend calcula a fração `discount = clamp(p / 100, 0, 1)` para aquele item

#### Scenario: Linha com total zero no modo valor
- **WHEN** a modalidade do item é `R$` e `price × quantity <= 0`
- **THEN** o frontend mantém `discount = 0` para aquele item sem erro de divisão

#### Scenario: Alternância de modalidade preserva o desconto
- **WHEN** o operador alterna a modalidade de um item entre `%` e `R$` sem digitar novo valor
- **THEN** o desconto aplicado (fração armazenada) é preservado e reexibido convertido para a nova modalidade, sem alterar o `discounted_price` do item

#### Scenario: Indicador de máximo na modalidade ativa
- **WHEN** o campo de desconto de um item está focado OU `discount > 0`
- **THEN** o indicador de desconto máximo continua exibindo o limite em R$ e em % (`· máx. R$ X (Y%)`), com ênfase na unidade da modalidade ativa

### Requirement: Desconto global rateado

O sistema SHALL permitir, na tela de fechamento, aplicar um desconto no total do pedido, distribuído proporcionalmente pelo valor de cada item.

#### Scenario: Desconto global em R$
- **WHEN** o operador informa um valor de desconto global em R$ no fechamento
- **THEN** o frontend calcula `peso_i = (price_i × qty_i) / total_bruto` e `discount_pct_i = (desconto_global × peso_i) / (price_i × qty_i)` para cada item, atualizando os descontos individuais

#### Scenario: Desconto global em %
- **WHEN** o operador informa um percentual de desconto global no fechamento
- **THEN** o frontend aplica o mesmo percentual a todos os itens (`discount_pct_i = percentual`)

#### Scenario: Rateio com item abaixo do piso
- **WHEN** o rateio do desconto global resulta em ao menos um item com `discounted_price < floor`
- **THEN** o sistema exibe aviso identificando quais itens ultrapassaram o piso, sem bloquear o fechamento

### Requirement: Seleção de modalidade e parcela no fechamento

O sistema SHALL permitir, no fechamento, escolher para cada pagamento o método e — quando o método tiver taxas por parcela — o número de parcelas. A UI do recebimento NÃO SHALL exibir a porcentagem da taxa nas opções de parcela nem o valor líquido recebido; a parcela escolhida continua determinando a taxa registrada no backend.

#### Scenario: Método com tabela de parcelas
- **WHEN** o operador seleciona um método que possui taxas configuradas
- **THEN** o sistema exibe as opções de parcela apenas pelo número de parcelas (ex.: "1x", "2x"), sem a porcentagem da taxa em cada opção

#### Scenario: Método sem parcelamento
- **WHEN** o operador seleciona um método sem taxa (ex.: Pix, dinheiro)
- **THEN** o sistema não exibe seletor de parcelas e usa `installments = 1`

#### Scenario: Valor líquido não exibido no recebimento
- **WHEN** o operador informa um valor para um pagamento com método que possui taxas
- **THEN** o sistema não exibe o valor líquido recebido ("você recebe") na tela de recebimento

#### Scenario: Parcela escolhida define a taxa registrada
- **WHEN** o operador confirma o pagamento com uma parcela escolhida
- **THEN** o `installments` escolhido é enviado ao `close_sale` e determina a taxa aplicada ao `net_amount`

### Requirement: Fechamento de pedido com pagamento

O sistema SHALL registrar o pagamento de um pedido — incluindo os descontos aplicados no fechamento, a modalidade/parcelas de cada pagamento e a taxa retida — e mudar seu status para `closed`, disparando as ordens de produção e consumindo automaticamente o estoque dos itens vendidos. Ao fechar, o sistema SHALL aplicar os descontos informados aos itens, recalcular o `total_value` e validar o pagamento contra esse total descontado, tudo em uma transação. O consumo de estoque segue a regra: se o item vendável possui componentes em `Product_Components`, consome os insumos da ficha técnica; caso contrário, consome o próprio produto (produto de revenda). A emissão de avisos de estoque insuficiente SHALL ser condicionada ao parâmetro `allow_negative_stock_sale`: quando `'false'` (padrão), o sistema emite avisos; quando `'true'`, o sistema prossegue silenciosamente. Em ambos os casos a venda prossegue e a baixa de estoque ocorre.

#### Scenario: Aplicação de desconto no fechamento
- **WHEN** o command `close_sale` é invocado com a lista de itens contendo `discount` por item
- **THEN** o sistema, para cada item, atualiza `Sales_Products.discount` e `Sales_Products.discounted_price = price × (1 − discount)`, recalcula `Sales.total_value = SUM(discounted_price × quantity)` e usa esse total para validar o pagamento, tudo na mesma transação

#### Scenario: Fechamento com pagamento único
- **WHEN** o command `close_sale` é invocado com os itens descontados e um único `Sale_Payment` cujo `amount_paid = total_value` recalculado, informando `payment_method_id` e `installments`
- **THEN** o sistema insere o registro em `Sale_Payments` com `net_amount = amount_paid × (1 − fee_percent) − fixed_fee` da taxa correspondente a `(payment_method_id, installments)`, grava o snapshot da taxa aplicada, cria `Production_Orders` para cada item com `product.sellable = true`, consome o estoque via `Stock_Movements`, popula `Production_Inputs` quando há componentes e atualiza `Sales.status = 'closed'`, tudo em uma transação

#### Scenario: Fechamento com split payment
- **WHEN** o command `close_sale` é invocado com os itens descontados e múltiplos `Sale_Payments` cuja soma de `amount_paid = total_value` recalculado, cada um com seu `payment_method_id` e `installments`
- **THEN** o sistema insere todos os registros em `Sale_Payments`, cada um com seu `net_amount` calculado pela taxa da respectiva parcela, cria `Production_Orders`, consome o estoque e popula `Production_Inputs` quando há componentes, tudo em uma transação

#### Scenario: Cálculo de net_amount com taxa percentual e fixa
- **WHEN** a parcela escolhida possui `fee_percent` e/ou `fixed_fee` configurados
- **THEN** `net_amount = amount_paid × (1 − fee_percent) − fixed_fee`, e a taxa aplicada (`fee_percent`, `fixed_fee`) é gravada como snapshot no `Sale_Payment`

#### Scenario: Forma de pagamento sem taxa para a parcela
- **WHEN** o método não possui taxa cadastrada para o `installments` informado (ex.: dinheiro, Pix)
- **THEN** `fee_percent = 0` e `fixed_fee = 0`, resultando em `net_amount = amount_paid`

#### Scenario: Pagamento com valor insuficiente
- **WHEN** a soma dos `amount_paid` é menor que o `total_value` recalculado com os descontos do fechamento
- **THEN** o sistema retorna erro de validação sem alterar nenhum registro

#### Scenario: Fechamento de pedido já fechado
- **WHEN** o command `close_sale` é invocado com `id` de pedido `closed`
- **THEN** o sistema retorna erro sem alterar nenhum registro

#### Scenario: Production_Orders e Production_Inputs criados no fechamento
- **WHEN** o pedido é fechado com sucesso e um item possui componentes definidos
- **THEN** o sistema cria um `Production_Order` para cada `Sales_Products` com `sellable = true`; para cada componente do produto cria um `Stock_Movement (type='out', motive='venda', quantity = component.quantity × sale_quantity)` e um `Production_Input (production_order_id, product_id=component.child_id, quantity_used)`

#### Scenario: Item vendável sem componentes baixa o próprio estoque
- **WHEN** o pedido é fechado e um item vendável não possui componentes em `Product_Components`
- **THEN** o sistema cria o `Production_Order` normalmente e insere um `Stock_Movement (type='out', motive='venda', product_id = item.product_id, quantity = sale_quantity)` baixando o saldo do próprio produto, sem inserir `Production_Inputs` para esse item

#### Scenario: Estoque insuficiente de insumo com avisos habilitados
- **WHEN** ao fechar a venda, a quantidade disponível de um insumo em `Stock` é menor que a quantidade a consumir e `allow_negative_stock_sale = 'false'`
- **THEN** o sistema prossegue com o fechamento e o consumo (estoque fica negativo), e retorna uma lista de avisos identificando os insumos com estoque insuficiente para exibição no frontend

#### Scenario: Estoque insuficiente de produto de revenda com avisos habilitados
- **WHEN** ao fechar a venda, um item vendável sem componentes tem `Stock.current_quantity` menor que `sale_quantity` e `allow_negative_stock_sale = 'false'`
- **THEN** o sistema prossegue com o fechamento e a baixa (estoque fica negativo), e inclui na lista de avisos o produto com estoque insuficiente

#### Scenario: Venda com estoque negativo permitida sem avisos
- **WHEN** ao fechar a venda, um ou mais itens/insumos têm estoque insuficiente e `allow_negative_stock_sale = 'true'`
- **THEN** o sistema prossegue com o fechamento e a baixa (estoque fica negativo) sem incluir avisos de estoque insuficiente na lista retornada

### Requirement: Listagem de métodos de pagamento

O sistema SHALL retornar os métodos de pagamento ativos para seleção no fechamento do pedido, cada um com sua tabela de taxas por número de parcelas.

#### Scenario: Métodos disponíveis
- **WHEN** o command `list_payment_methods` é invocado
- **THEN** o sistema retorna todos os registros de `Payment_Methods` com `active = true`, incluindo `id`, `name` e a lista de taxas (`installments`, `fee_percent`, `fixed_fee`) de cada método

#### Scenario: Nenhum método cadastrado
- **WHEN** não há registros em `Payment_Methods`
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Listagem de pagamentos de um pedido

O sistema SHALL retornar, somente para consulta, os pagamentos registrados de um pedido, cada um com o nome do método de pagamento, o valor pago e o valor líquido. O command NÃO SHALL alterar nenhum registro.

#### Scenario: Pedido com pagamentos
- **WHEN** o command `get_sale_payments` é invocado com um `sale_id` que possui registros em `Sale_Payments`
- **THEN** o sistema retorna, para cada pagamento, o `payment_method_name` (de `Payment_Methods`), o `amount_paid` e o `net_amount`, ordenados por `created_at`

#### Scenario: Pedido sem pagamentos
- **WHEN** o command `get_sale_payments` é invocado com um `sale_id` sem registros em `Sale_Payments` (ex.: pedido em aberto)
- **THEN** o sistema retorna lista vazia sem erro

### Requirement: Visualização somente leitura de pedido fechado

O sistema SHALL permitir abrir, a partir do histórico, um recibo somente leitura de um pedido fechado, exibindo seus itens com os descontos aplicados, o total da venda e os pagamentos. A visualização NÃO SHALL oferecer edição dos dados do pedido.

#### Scenario: Ação de visualização no histórico
- **WHEN** a lista de pedidos fechados é exibida no histórico
- **THEN** cada pedido apresenta uma ação "Ver" que abre o recibo somente leitura daquele pedido

#### Scenario: Conteúdo do recibo
- **WHEN** o recibo de um pedido fechado é aberto
- **THEN** o sistema exibe, para cada item, o `price` (preço cheio), o desconto aplicado e o `discounted_price × quantity` (subtotal descontado), o `total_value` da venda, e a lista de pagamentos com a forma de pagamento e o valor de cada um

#### Scenario: Item sem desconto
- **WHEN** um item do pedido possui `discount = 0`
- **THEN** o recibo exibe o subtotal sem indicar desconto para aquele item

#### Scenario: Ausência de edição
- **WHEN** o recibo de um pedido fechado está aberto
- **THEN** a tela não exibe campos editáveis nem botões de salvar, editar ou fechar a venda
