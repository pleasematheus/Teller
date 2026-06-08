## Purpose

Secao "Vendas" do modulo de Configuracoes. Agrupa parametros de comportamento das vendas, comecando pelo controle de bloqueio de venda com estoque negativo.

## Requirements

### Requirement: Secao Vendas nas Configuracoes
O modulo de Configuracoes SHALL exibir uma secao "Vendas" no sidebar de navegacao, posicionada antes da secao "Sistema".

#### Scenario: Aba Vendas presente e ordenada
- **WHEN** o usuario abre o modulo de Configuracoes
- **THEN** existe um item de navegacao "Vendas" no sidebar
- **AND** o item "Vendas" aparece antes do item "Sistema"

### Requirement: Bloqueio de venda com estoque negativo na secao Vendas
A secao "Vendas" SHALL conter o controle que define o parametro `allow_negative_stock_sale`, com o mesmo comportamento de leitura e gravacao previamente disponivel na secao "Sistema".

#### Scenario: Switch exibido na secao Vendas
- **WHEN** o usuario seleciona a secao "Vendas"
- **THEN** o switch "Bloquear vendas com estoque negativo" e exibido
- **AND** seu estado reflete o valor atual de `allow_negative_stock_sale`

#### Scenario: Alteracao persiste
- **WHEN** o usuario alterna o switch na secao "Vendas"
- **THEN** o parametro `allow_negative_stock_sale` e gravado via `set_system_setting`
- **AND** ao reabrir as Configuracoes o switch reflete o valor gravado

#### Scenario: Switch ausente da secao Sistema
- **WHEN** o usuario seleciona a secao "Sistema"
- **THEN** o switch "Bloquear vendas com estoque negativo" NAO e exibido nessa secao
