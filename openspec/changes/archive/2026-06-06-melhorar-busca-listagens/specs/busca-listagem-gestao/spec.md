## ADDED Requirements

### Requirement: Barra de ferramentas de listagem reutilizável

O sistema SHALL fornecer um componente de barra de ferramentas compartilhado para as telas de gestão, composto por um campo de busca por texto, um grupo de filtros segmentados e um slot opcional de ação. O componente SHALL ser UI pura: emite as mudanças de busca e de filtro via callbacks e NÃO SHALL conter lógica de negócio nem renderizar a lista ou tabela de itens, que permanece responsabilidade da tela dona.

#### Scenario: Barra com busca, filtros e ação
- **WHEN** uma tela renderiza a barra passando filtros e um nó de ação
- **THEN** a barra exibe o campo de busca, os filtros segmentados e a ação no slot designado

#### Scenario: Barra sem slot de ação
- **WHEN** uma tela renderiza a barra sem passar nó de ação
- **THEN** a barra exibe apenas o campo de busca e os filtros, sem espaço de ação

#### Scenario: Mudança de busca emitida
- **WHEN** o usuário digita no campo de busca
- **THEN** a barra emite o novo texto via callback sem filtrar ou ordenar itens por conta própria

#### Scenario: Mudança de filtro emitida
- **WHEN** o usuário seleciona um filtro segmentado
- **THEN** a barra emite o filtro escolhido via callback e marca visualmente o filtro ativo

### Requirement: Busca pura por nome para listas de gestão

O sistema SHALL fornecer uma função de busca pura que filtra e ordena qualquer lista de itens que possuam um campo `name`, ignorando diferenças de acentuação e de caixa, priorizando correspondências por prefixo sobre correspondências em outras posições. A lógica SHALL ser isolada da camada de UI, reutilizar a mesma normalização de acentos da busca de produto existente (sem duplicá-la) e ser coberta por teste.

#### Scenario: Correspondência ignora acento
- **WHEN** o usuário digita "acai" e existe um item chamado "Açaí 300ml"
- **THEN** "Açaí 300ml" aparece nos resultados

#### Scenario: Prefixo ranqueado acima de substring
- **WHEN** o usuário digita "co" e existem os itens "Coca" (prefixo) e "Chocolate" (substring no meio)
- **THEN** "Coca" aparece antes de "Chocolate" nos resultados

#### Scenario: Busca vazia preserva a lista
- **WHEN** o texto de busca está vazio
- **THEN** a função retorna a lista original sem filtrar nem reordenar
