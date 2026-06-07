## 1. Lógica de busca pura genérica

- [x] 1.1 Exportar `normalize` de `src/lib/productSearch.ts` (tornar fonte única de remoção de diacríticos)
- [x] 1.2 Criar `src/lib/listSearch.ts` com `matchesByName<T extends { name: string }>(item, query)` e `rankByName<T extends { name: string }>(items, query)` (prefixo no name > substring no name), importando `normalize` de `productSearch.ts`
- [x] 1.3 Criar `src/lib/listSearch.test.ts` (Vitest) cobrindo: insensível a acento, match por substring, ranqueamento prefixo > substring, query vazia retorna a lista original
- [x] 1.4 Spawnar subagente para rodar `bun run test` e reportar resultado dos testes de `listSearch`

## 2. Barra de ferramentas reutilizável

- [x] 2.1 Criar `src/components/ui/ListToolbar.tsx` (UI pura) com props `{ query, onQuery, filters, activeFilter, onFilter, searchPlaceholder?, action? }`: input de busca com ícone, filtros segmentados e slot de ação, usando tokens `teller-*`
- [x] 2.2 Garantir que a barra não contém lógica de negócio nem renderiza lista (só emite estado via callbacks)

## 3. Busca e filtro na listagem de produtos (ProductList)

- [x] 3.1 Adicionar estado de busca em `ProductList.tsx` e renderizar `ListToolbar` com os filtros existentes (Todos/Vendáveis/Insumos) e o botão "Novo produto" no slot `action`
- [x] 3.2 Compor a lista exibida: filtrar por tipo e então `rankProducts(filtrados, query)` (name + description, acento-insensível, ranqueado)
- [x] 3.3 Exibir estado vazio distinto: "nenhum resultado" quando há produtos mas a busca/filtro não casa, separado de "nenhum produto cadastrado"

## 4. Busca e filtro no saldo de estoque (StockBalance)

- [x] 4.1 Em `StockBalance.tsx`, derivar `Set<number>` de `product_id` vendáveis a partir do `list_products` já carregado
- [x] 4.2 Adicionar estado de busca e filtro e renderizar `ListToolbar` (sem slot de ação) com os filtros Todos/Vendáveis/Insumos
- [x] 4.3 Compor a tabela exibida: filtrar por tipo (via set de vendáveis) e então `rankByName(filtrados, query)` por nome
- [x] 4.4 Exibir estado vazio distinto: "nenhum resultado" quando há estoque mas a busca/filtro não casa, separado de "nenhum produto cadastrado"

## 5. Verificação

- [x] 5.1 Spawnar subagente para rodar `bun run test` e `tsc` (build) e reportar erros
- [x] 5.2 [Teste manual - tester humano] Produtos: buscar com e sem acento, alternar filtros, combinar busca + filtro, conferir ranqueamento (prefixo no topo), estado vazio de resultado e botão "Novo produto" funcional
- [x] 5.3 [Teste manual - tester humano] Estoque: buscar por nome, alternar Todos/Vendáveis/Insumos, conferir que o tipo bate com o cadastro do produto, combinar busca + filtro, estado vazio de resultado
