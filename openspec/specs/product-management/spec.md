## Purpose

Listagem, cadastro e edição de produtos com distinção entre insumos e produtos vendáveis, incluindo precificação automática via fórmula CMV × Markup.

## Requirements

### Requirement: Listagem de produtos

O sistema SHALL retornar todos os produtos cadastrados com seus dados de precificação, ordenados por nome, incluindo os campos `sellable` e `unit` para uso no cliente.

#### Scenario: Lista com produtos existentes
- **WHEN** o comando `list_products` é invocado
- **THEN** o sistema retorna todos os registros de `Products` com campos: id, name, description, active, sellable, cost_price, markup, final_price, expected_loss_rate, **unit**

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

O sistema SHALL permitir criar um novo produto com nome, descrição opcional, flags `active` e `sellable`, custo, taxa de perda e **unidade de medida**.

#### Scenario: Cadastro com dados válidos
- **WHEN** o usuário submete nome não-vazio, `cost_price >= 0`, `expected_loss_rate` entre 0 e 1 e `unit` válido
- **THEN** o sistema insere o produto em `Products` com `markup` e `final_price` calculados e retorna sucesso

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

O sistema SHALL permitir atualizar os dados de um produto existente incluindo `unit`, recalculando `markup` e `final_price` ao salvar.

#### Scenario: Edição com dados válidos
- **WHEN** o usuário edita um produto existente e submete dados válidos incluindo `unit`
- **THEN** o sistema atualiza o registro em `Products` com os novos valores e `updated_at` atualizado

#### Scenario: Produto inexistente
- **WHEN** o comando `update_product` é invocado com `id` que não existe
- **THEN** o sistema retorna erro sem alterar nenhum registro

### Requirement: Precificação automática no formulário

O sistema SHALL calcular e exibir em tempo real CMV, Markup% e Preço Final conforme o usuário edita `cost_price` e `expected_loss_rate`, usando as taxas atuais de `get_pricing_rates`.

#### Scenario: Cálculo com histórico de vendas disponível
- **WHEN** o formulário é aberto e existem vendas nos últimos 12 meses
- **THEN** CV% e DF% são derivados de `Financial_Entries` / faturamento dos últimos 12 meses

#### Scenario: Cálculo sem histórico de vendas
- **WHEN** o formulário é aberto e não existem vendas nos últimos 12 meses
- **THEN** CV% = 0 e DF% = 0, e o formulário exibe aviso de que as taxas não puderam ser calculadas

#### Scenario: Atualização em tempo real
- **WHEN** o usuário altera `cost_price` ou `expected_loss_rate`
- **THEN** CMV, Markup% e Preço Final são recalculados instantaneamente no cliente sem nova requisição

### Requirement: Cálculo automático de taxa de perda

O sistema SHALL calcular `expected_loss_rate` a partir do histórico de `Stock_Movements` do produto quando o usuário solicitar, como alternativa ao preenchimento manual.

#### Scenario: Cálculo com histórico disponível
- **WHEN** o usuário aciona "Calcular" para perda automática em um produto existente
- **THEN** o sistema retorna `SUM(quantity WHERE movement_type='out' AND motive='perda') / SUM(quantity WHERE movement_type='in')` para aquele produto

#### Scenario: Cálculo sem histórico de movimentos
- **WHEN** o produto não possui `Stock_Movements`
- **THEN** o sistema retorna 0 sem erro

#### Scenario: Modo auto indisponível em criação
- **WHEN** o formulário é aberto para criação de novo produto
- **THEN** a opção de cálculo automático de perda está desabilitada (produto ainda não tem histórico)
