## ADDED Requirements

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
