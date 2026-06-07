## ADDED Requirements

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
