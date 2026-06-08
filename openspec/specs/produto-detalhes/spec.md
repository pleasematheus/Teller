## Purpose

Sub-aba de informacoes detalhadas dentro da tela de edicao de produto, consolidando historico de vendas, movimentacoes de estoque, historico de compras e detalhamento de precificacao em um unico lugar sem trocar de modulo.

## Requirements

### Requirement: Sub-aba de detalhes do produto

A tela de produto SHALL exibir, **apenas no modo edicao** (produto ja persistido), duas sub-abas internas: "Geral" (o formulario de cadastro/precificacao existente) e "Detalhes". No modo "Novo produto" nenhuma sub-aba SHALL aparecer; apenas o formulario e exibido. A sub-aba "Detalhes" SHALL reunir, para o produto selecionado, as secoes de historico de vendas, movimentacoes de estoque, historico de compras e detalhamento de precificacao. A sub-aba "Detalhes" SHALL ser **estritamente de leitura**: nao expoe campos editaveis, botoes de salvar/cadastrar nem qualquer acao de escrita; toda edicao do produto permanece exclusivamente na aba "Geral".

#### Scenario: Sub-abas no modo edicao
- **WHEN** o usuario abre um produto existente para edicao
- **THEN** a area do formulario exibe as sub-abas "Geral" e "Detalhes", com "Geral" ativa por padrao

#### Scenario: Sem sub-abas no modo novo
- **WHEN** o usuario inicia o cadastro de um novo produto
- **THEN** nenhuma sub-aba e exibida e apenas o formulario de cadastro aparece

#### Scenario: Detalhes e somente leitura
- **WHEN** o usuario abre a sub-aba "Detalhes"
- **THEN** todo o conteudo e apresentado como leitura (tabelas e valores), sem campos editaveis, sem botao de salvar e sem acoes que alterem dados

#### Scenario: Alternancia preserva o formulario
- **WHEN** o usuario edita campos na aba "Geral", abre "Detalhes" e volta para "Geral"
- **THEN** os valores digitados na aba "Geral" sao preservados

### Requirement: Historico de vendas por produto

O sistema SHALL fornecer um comando `get_product_sales_history(product_id)` que retorna as vendas **fechadas** que incluem o produto, com data, quantidade vendida, preco unitario e receita por venda, ordenadas por data decrescente. A sub-aba "Detalhes" SHALL exibir essas linhas na secao "Historico de vendas".

#### Scenario: Produto com vendas
- **WHEN** `get_product_sales_history` e invocado para um produto que aparece em vendas fechadas
- **THEN** o sistema retorna uma linha por venda com data, quantidade, preco unitario e receita, ordenadas da mais recente para a mais antiga

#### Scenario: Produto sem vendas
- **WHEN** o produto nao aparece em nenhuma venda fechada
- **THEN** o comando retorna lista vazia e a secao exibe mensagem de "nenhuma venda registrada"

#### Scenario: Rascunhos de venda nao entram no historico
- **WHEN** existe uma venda em rascunho (`status` diferente de fechada) contendo o produto
- **THEN** essa venda NAO aparece no historico de vendas

### Requirement: Movimentacoes de estoque na sub-aba de detalhes

A sub-aba "Detalhes" SHALL exibir as movimentacoes de estoque do produto reutilizando o comando existente `list_stock_movements(product_id)`, mostrando data, tipo (entrada/saida), motivo e quantidade, sem exigir que o usuario selecione o produto manualmente.

#### Scenario: Movimentacoes do produto em contexto
- **WHEN** a secao de movimentacoes e renderizada para um produto em edicao
- **THEN** o sistema carrega `list_stock_movements` para aquele `product_id` automaticamente e lista as movimentacoes

#### Scenario: Produto sem movimentacoes
- **WHEN** o produto nao possui movimentacoes registradas
- **THEN** a secao exibe mensagem de "nenhuma movimentacao registrada"

### Requirement: Historico de compras por produto

O sistema SHALL fornecer um comando `list_product_purchases(product_id)` que retorna as compras **fechadas** que incluem o produto/insumo, com data, referencia da compra, quantidade, preco unitario e total, ordenadas por data decrescente. A sub-aba "Detalhes" SHALL exibir essas linhas na secao "Historico de compras".

#### Scenario: Insumo com compras
- **WHEN** `list_product_purchases` e invocado para um insumo presente em compras fechadas
- **THEN** o sistema retorna uma linha por compra com data, referencia da compra, quantidade, preco unitario e total, ordenadas da mais recente para a mais antiga

#### Scenario: Produto sem compras
- **WHEN** o produto nao aparece em nenhuma compra fechada
- **THEN** o comando retorna lista vazia e a secao exibe mensagem de "nenhuma compra registrada"

### Requirement: Detalhamento de precificacao na sub-aba de detalhes

A sub-aba "Detalhes" SHALL exibir o detalhamento completo de precificacao do produto reutilizando a logica de `src/lib/pricing.ts`, apresentando CMV, taxas (CV, DF, investimento), margem de lucro, markup e preco final. O calculo SHALL ser consistente com o exibido na aba "Geral" (mesma funcao pura, sem duplicacao de formula).

#### Scenario: Breakdown de produto precificado
- **WHEN** a secao de precificacao e renderizada para um produto com preco final definido
- **THEN** o sistema exibe CMV, CV, DF, investimento, margem, markup e preco final calculados a partir dos dados do produto e das `PricingRates`

#### Scenario: Consistencia com a aba Geral
- **WHEN** o mesmo produto e visto na aba "Geral" e na secao de precificacao de "Detalhes"
- **THEN** os valores de CMV e margem coincidem
