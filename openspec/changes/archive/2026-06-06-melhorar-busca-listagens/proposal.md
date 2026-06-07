## Why

As telas de gestão de **Produtos** (`ProductList`) e **Estoque** (`StockBalance`) não têm busca por texto. O operador só consegue localizar um item rolando a lista inteira: em `ProductList` há apenas os filtros por tipo (Todos/Vendáveis/Insumos) e em `StockBalance` não há nem busca nem filtro, só a tabela crua. O combobox de busca já existe (`ProductCommand` + `src/lib/productSearch.ts`), mas atende apenas os fluxos transacionais de venda e compra; as telas de listagem ficaram de fora. Para um fluxo de balcão com catálogo crescente, localizar um item por digitação é o caminho esperado.

## What Changes

- Nova barra de ferramentas de listagem reutilizável `src/components/ui/ListToolbar.tsx`: input de busca + filtros segmentados + slot opcional de ação (ex.: botão "Novo produto"). É UI pura, sem lógica de negócio nem renderização de lista; a tela dona renderiza sua própria lista/tabela.
- Lógica de busca pura genérica em `src/lib/listSearch.ts`: `matchesByName` e `rankByName` operando sobre qualquer item com `name` (serve `StockItem`, que não tem `description`). Reaproveita `normalize` (acento/caixa) de `productSearch.ts` como fonte única, sem duplicar.
- `ProductList`: adiciona a `ListToolbar` com o input de busca, os filtros existentes (Todos/Vendáveis/Insumos) e o botão "Novo produto" no slot de ação. Compõe filtro por tipo e depois ranqueia por busca (`rankProducts`, name + description). Estado vazio distinto para "nenhum resultado" versus "nenhum produto cadastrado".
- `StockBalance`: adiciona a `ListToolbar` com input de busca e filtros Todos/Vendáveis/Insumos. Como `StockItem` não traz `sellable`, deriva o conjunto de ids vendáveis cruzando com `list_products` (já carregado na tela). Compõe filtro e depois ranqueia por nome (`rankByName`). Estado vazio distinto para "nenhum resultado".
- Ordenação ranqueada (melhor correspondência no topo) em ambas as telas, reaproveitando o ranqueamento existente.
- Sem alteração de banco e sem novos comandos Tauri: a busca opera sobre dados já retornados por `list_products` e `list_stock`.

## Capabilities

### New Capabilities
- `busca-listagem-gestao`: barra de ferramentas de listagem reutilizável (busca por texto + filtros segmentados + ação) e busca pura por nome para as telas de gestão, com ranqueamento e estado vazio explícito.

### Modified Capabilities
- `product-management`: a listagem de produtos passa a oferecer busca por texto (nome/descrição, insensível a acento, ranqueada) combinada com os filtros por tipo já existentes.
- `stock-management`: a consulta de saldo de estoque passa a oferecer busca por nome e filtro por tipo (Todos/Vendáveis/Insumos), com o tipo derivado do cruzamento com `list_products`.

## Impact

- Código: novos `src/components/ui/ListToolbar.tsx`, `src/lib/listSearch.ts` (+ `.test.ts`); editados `src/components/products/ProductList.tsx`, `src/components/stock/StockBalance.tsx`; `src/lib/productSearch.ts` exporta `normalize` para reuso.
- Dependências: nenhuma nova.
- Banco de dados: nenhum (sem migração DuckDB).
- Comandos Tauri: nenhum novo; `list_products` e `list_stock` permanecem inalterados.
