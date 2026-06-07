## ADDED Requirements

### Requirement: Busca por texto na listagem de produtos

A tela de listagem de produtos SHALL oferecer um campo de busca por texto que filtra os produtos por correspondência no `name` e na `description`, ignorando acentuação e caixa, e SHALL ordenar os resultados priorizando correspondências por prefixo. A busca SHALL operar no cliente sobre os dados já retornados por `list_products`, sem nova requisição ao backend, e SHALL compor com o filtro por tipo (Todos/Vendáveis/Insumos) já existente: o filtro recorta o conjunto e a busca ranqueia o subconjunto.

#### Scenario: Busca por nome insensível a acento
- **WHEN** o usuário digita "acai" na busca e existe um produto "Açaí 300ml"
- **THEN** a lista exibe "Açaí 300ml"

#### Scenario: Busca por descrição
- **WHEN** o termo digitado não casa com nenhum `name` mas casa com a `description` de um produto
- **THEN** esse produto aparece na lista

#### Scenario: Busca combinada com filtro de tipo
- **WHEN** o filtro "Vendáveis" está ativo e o usuário digita um termo de busca
- **THEN** a lista exibe apenas produtos vendáveis cujo nome ou descrição casa com o termo

#### Scenario: Ranqueamento por prefixo
- **WHEN** o usuário digita "co" e existem "Coca" (prefixo) e "Chocolate" (substring)
- **THEN** "Coca" aparece antes de "Chocolate" na lista

#### Scenario: Resultado vazio distinto de catálogo vazio
- **WHEN** existem produtos cadastrados mas nenhum casa com a busca ou o filtro ativo
- **THEN** a tela exibe uma mensagem de "nenhum resultado", distinta da mensagem de catálogo sem produtos cadastrados

#### Scenario: Busca vazia preserva a listagem filtrada
- **WHEN** o campo de busca está vazio
- **THEN** a lista exibe todos os produtos do filtro de tipo ativo, sem reordenação por busca
