## Context

Há busca de produto no projeto, mas só no fluxo transacional:

- `src/lib/productSearch.ts` expõe `normalize`, `matchesProduct`, `rankProducts(products, query)` e `scoreProduct(product, query)`, operando sobre `Product` (campos `name` + `description`). Tem teste Vitest (`productSearch.test.ts`).
- `src/components/ui/ProductCommand.tsx` é o combobox (cmdk) usado por venda e compra. Usa `scoreProduct` como `filter`.

As telas de gestão ficaram sem busca:

- `ProductList.tsx` tem só os filtros por tipo (`all`/`sellable`/`inputs`) via `useState` + `.filter`, e o botão "Novo produto". Renderiza cards. Não há input de busca.
- `StockBalance.tsx` renderiza uma `<table>` de `StockItem` (`product_id`, `name`, `unit`, `current_quantity`). Já invoca `list_stock` **e** `list_products` (este último para montar o mapa de capacidade produtiva). Não há busca nem filtro.

O modelo `StockItem` **não** tem `description` nem `sellable`. A busca de estoque opera só por `name`; o filtro por tipo exige derivar `sellable` cruzando `product_id` com os `Product` de `list_products`.

## Goals / Non-Goals

**Goals:**
- Uma barra de ferramentas única (`ListToolbar`) reutilizada nas duas telas: input de busca + filtros segmentados + slot de ação opcional.
- Lógica de busca pura e testável que sirva tanto `Product` (name+description) quanto `StockItem` (name).
- Busca insensível a acento/caixa e ranqueada (prefixo > substring) em ambas as telas.
- Filtro Todos/Vendáveis/Insumos nas duas telas, com o tipo do estoque derivado de `list_products`.

**Non-Goals:**
- Busca por código de barras / SKU (campo não existe no modelo `Product`; fora de escopo, igual à mudança anterior).
- Componente que também renderize a lista/tabela (cards de produto e tabela de estoque divergem demais; a `ListToolbar` é só a barra).
- Filtros de estoque baseados em regra nova (ex.: "estoque baixo"/"zerado"), que exigiriam threshold; fora de escopo.
- Qualquer alteração de comando Tauri ou schema DuckDB.

## Decisions

**1. `ListToolbar` é UI pura; cada tela mantém sua lista.**
O usuário optou por abstrair só a barra. `ProductList` usa cards e `StockBalance` usa tabela; unificar a renderização forçaria um `renderItem` complexo sem ganho. Assinatura aproximada:

```
ListToolbar({
  query, onQuery,
  filters,          // { value, label }[]
  activeFilter, onFilter,
  searchPlaceholder?,
  action?,          // ReactNode (ex.: <Button>Novo produto</Button>)
})
```

A barra emite mudanças de estado; a tela faz `filter` + `rank` e renderiza. Sem estado de negócio dentro da barra, testável e reaproveitável.

**2. `listSearch.ts` genérico por `name`, `normalize` com fonte única.**
Funções `matchesByName<T extends { name: string }>(item, query)` e `rankByName<T extends { name: string }>(items, query)` (prefixo no name > substring no name). Serve `StockItem` e qualquer lista nomeada. `normalize` não é reescrito: `productSearch.ts` passa a **exportar** `normalize` e `listSearch.ts` o importa (uma definição só de remoção de diacríticos). `productSearch.ts` (name+description) continua existindo para `ProductList` e `ProductCommand`.

**3. Composição filtro → ranqueamento.**
Primeiro aplica o filtro por tipo (recorta o conjunto), depois ranqueia por busca o subconjunto. Ordem escolhida para que a contagem de "nenhum resultado" reflita o filtro ativo e a busca opere só sobre o que o filtro deixou passar.
- `ProductList`: `rankProducts(produtosFiltradosPorTipo, query)` (name + description).
- `StockBalance`: `rankByName(estoqueFiltradoPorTipo, query)` (name).

**4. Tipo no estoque derivado de `list_products`.**
`StockBalance` já carrega `list_products`. Constrói um `Set<number>` de `product_id` vendáveis (`p.sellable`) e usa `item.product_id` para classificar cada linha. Filtro "Vendáveis" mantém ids no set; "Insumos" os de fora; "Todos" não filtra. Nenhuma chamada nova ao backend.

**5. Ordenação ranqueada em ambas, inclusive na tabela.**
A tabela de estoque reordena conforme a digitação (melhor match no topo). As listas são curtas e o objetivo é localizar rápido; aceita-se o reposicionamento. Caso o reposicionamento incomode em uso real, a alternativa é filtro de ordem estável (esconder não-correspondentes mantendo a sequência), troca local sem mudança de contrato.

**6. Estado vazio distinto de "sem dados".**
Hoje `ProductList` tem "Nenhum produto encontrado" e `StockBalance` tem "Nenhum produto cadastrado" (zero dados). Com busca/filtro, é preciso distinguir lista-fonte vazia (sem cadastro) de resultado-vazio (busca/filtro sem casar). Cada tela exibe a mensagem adequada conforme a fonte tem itens mas o resultado é vazio.

## Risks / Trade-offs

- [Reordenação ranqueada na tabela de estoque pode parecer instável ao digitar] -> Mitigação: listas curtas; decisão 5 prevê troca para ordem estável se necessário, sem mudar contrato.
- [Duplicar `normalize` entre `productSearch` e `listSearch`] -> Evitado: `normalize` exportado de um único módulo e importado pelo outro.
- [Derivar `sellable` no estoque depende de `list_products` retornar o produto] -> `StockBalance` já depende de `list_products`; item de estoque sem produto correspondente (caso de borda) cai em "Insumo" por ausência no set; coberto por cenário.

## Migration Plan

Sem migração de banco. Passos: exportar `normalize` de `productSearch.ts`; criar `listSearch.ts` + teste; criar `ListToolbar.tsx`; ligar em `ProductList` (busca + filtros existentes + ação) e em `StockBalance` (busca + filtros derivados). Rollback: reverter os arquivos de UI/lib; nada persistido muda.

## Open Questions

- Texto exato dos placeholders e rótulos da barra fica para a implementação; o spec só exige que exista input de busca e os filtros Todos/Vendáveis/Insumos.
