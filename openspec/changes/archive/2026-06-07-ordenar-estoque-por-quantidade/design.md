## Context

`StockBalance.tsx` já compõe filtro de tipo (`byType`) seguido de busca por nome (`rankByName(byType, query)`) para produzir `displayed`. A tabela tem 4 colunas de dados (Produto, Unidade, Quantidade, Capacidade estimada) mais uma de ação, todas estáticas. `listSearch.ts` reúne utilitários puros e genéricos (`matchesByName`, `rankByName`) cobertos por testes Vitest. Padrão do projeto: extrair lógica pura para `src/lib/` com teste correspondente.

A coluna Capacidade estimada é um texto derivado por linha: para cada uso do insumo, `Math.floor(current_quantity / qtyPerUnit)`. Pode haver vários usos (texto multi-valor) ou nenhum ("—").

## Goals / Non-Goals

**Goals:**
- Ordenar a listagem por qualquer coluna de dados (Produto, Unidade, Quantidade, Capacidade), asc/desc, com terceiro estado "sem ordenação".
- Apenas uma coluna ativa por vez.
- Manter a composição existente filtro -> busca -> ordenação como pipeline puro.
- Função de ordenação pura, genérica e testada.

**Non-Goals:**
- Ordenar a coluna de ação.
- Ordenação multi-coluna simultânea.
- Persistir a preferência de ordenação entre sessões.
- Mudanças no backend, `list_stock` ou schema.

## Decisions

### Estado de ordenação: coluna + direção, ciclo de 3 estados

```ts
type SortColumn = "name" | "unit" | "quantity" | "capacity"
type SortState = { column: SortColumn; dir: "asc" | "desc" } | null
const [sort, setSort] = useState<SortState>(null)
```

Clique num cabeçalho:
- coluna diferente da ativa (ou nenhuma ativa) -> `{ column, dir: "asc" }`
- mesma coluna em `asc` -> `desc`
- mesma coluna em `desc` -> `null` (volta ao padrão)

Alternativa considerada: dropdown de ordenação. Descartado - cabeçalho clicável é o padrão de tabela mais reconhecível e cobre todas as colunas sem UI extra.

### Função pura genérica em lib

Adicionar em `src/lib/listSearch.ts`:

```ts
export type SortDir = "asc" | "desc"

export function sortBy<T>(
  items: T[],
  value: (item: T) => string | number | null,
  dir: SortDir,
): T[] {
  const sign = dir === "asc" ? 1 : -1
  return [...items].sort((a, b) => {
    const va = value(a)
    const vb = value(b)
    if (va === null && vb === null) return 0
    if (va === null) return 1        // nulos sempre ao fim
    if (vb === null) return -1
    if (typeof va === "number" && typeof vb === "number") return (va - vb) * sign
    return String(va).localeCompare(String(vb), "pt-BR") * sign
  })
}
```

Recebe acessor de chave -> genérica e desacoplada de `StockItem`. Retorna novo array (imutável, `[...items].sort`). Nulos sempre ao fim independentemente da direção (regra da coluna Capacidade). Números comparados numericamente; demais via `localeCompare("pt-BR")` (colação sensível a acento). Alternativa: ordenar inline no componente - descartado, viola a regra de extrair lógica para `src/lib/` com teste.

### Acessores por coluna no componente

Mapa de acessores que extraem o valor de ordenação de cada `StockItem`:

```ts
const SORT_VALUE: Record<SortColumn, (item: StockItem) => string | number | null> = {
  name: (i) => i.name,
  unit: (i) => UNIT_LABEL[i.unit] ?? i.unit,
  quantity: (i) => i.current_quantity,
  capacity: (i) => {
    const usages = usageMap.get(i.product_id) ?? []
    if (!usages.length) return null
    return Math.max(...usages.map(({ qtyPerUnit }) => Math.floor(i.current_quantity / qtyPerUnit)))
  },
}
```

`capacity` retorna o MAIOR count da linha, ou `null` quando não há usos (vai ao fim). `usageMap` está no escopo do componente, então o acessor é montado no render (closure) e não no módulo lib.

### Precedência sobre o ranking de busca

Pipeline em `StockBalance`:
`byType` -> `base = rankByName(byType, query)` -> `displayed = sort ? sortBy(base, SORT_VALUE[sort.column], sort.dir) : base`.

`sort` do V8 é estável: com `sort = null` a ordem de `rankByName` (ou padrão) é preservada; com `sort` ativo a ordenação por coluna domina. Satisfaz "ordenação tem precedência sobre ranking de busca".

### Indicador visual no cabeçalho

Cada `<th>` de coluna ordenável vira/contém um `<button>` com seta (ícones Hugeicons, ArrowUp/ArrowDown) refletindo `sort` quando aquela coluna está ativa. Colunas inativas: sem seta (ou seta neutra discreta).

## Risks / Trade-offs

- [Empate de valores] -> `sort` estável preserva a ordem anterior (ranking de nome ou padrão), comportamento previsível.
- [Confusão entre ordenação e ranking de busca] -> documentado na spec: ordenação ativa sempre vence; remover ordenação volta ao ranking.
- [Capacidade multi-valor] -> decisão: ordenar pela MAIOR capacidade da linha; vazios ao fim. Simples e útil para priorizar reposição.
- [Acoplamento da função genérica] -> mitigado pelo acessor de chave; reutilizável em outras listas futuras.
