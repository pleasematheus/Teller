## MODIFIED Requirements

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
