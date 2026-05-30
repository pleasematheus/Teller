## Context

O módulo de produtos (`src/components/products/`) tem duas visões: a Lista (`ProductList`) e o formulário (`ProductForm`), orquestrados por `index.tsx` (`Products`). Clicar num produto abre o `ProductForm` em modo edição — esta é a "tela de produto" de um item.

O backend já expõe o saldo: `list_stock` (em `src-tauri/src/stock.rs`) faz `LEFT JOIN` de `Products` com `Stock` e retorna `StockItem { product_id, name, unit, current_quantity }` com `COALESCE(..., 0.0)` para todo produto. O tipo TS `StockItem` já existe em `src/types/stock.ts`. Logo, não há trabalho de backend nem de schema.

## Goals / Non-Goals

**Goals:**
- Mostrar `current_quantity` do produto em edição dentro do `ProductForm`, somente leitura.
- Reaproveitar `list_stock` e o tipo `StockItem` existentes.

**Non-Goals:**
- Nenhuma edição ou ajuste de estoque a partir do formulário.
- Nenhum saldo na criação de produto novo.
- Nenhuma mudança em `ProductList` (a tela de produto referida é o formulário).
- Nenhum comando Rust novo.

## Decisions

**Onde buscar o saldo: carregar em `Products` (index.tsx) e repassar por prop.**
`Products` já chama `list_products` e `get_pricing_rates` no mount. Adicionar `list_stock` ao mesmo fluxo e construir um `Map<number, number>` (product_id → current_quantity) é barato e mantém o `ProductForm` sem novo efeito de rede. Passa-se ao `ProductForm` apenas o número do saldo do produto em edição (`stockQuantity?: number`).
Alternativa considerada: `ProductForm` chamar `list_stock` por conta própria num `useEffect`. Rejeitada por adicionar mais uma chamada e duplicar carregamento já feito no container.

**Exibição: campo de leitura no padrão existente.**
Reaproveitar o mesmo estilo do campo de custo derivado já presente (div com `border`/`bg-teller-card`, `text-teller-muted`), exibindo `quantidade + unidade`. Formatar a quantidade com a mesma lógica de `StockBalance` (`fmtQty`: inteiro sem casas, senão 2 casas) para consistência visual entre telas.

**Condição de exibição: apenas em edição (`isEdit`/`product !== null`).**
Na criação não há `id` nem saldo; o campo é omitido.

## Risks / Trade-offs

- [Saldo defasado se o estoque mudar enquanto o formulário está aberto] → Aceitável: o saldo é informativo e recarrega ao reabrir; `onSaved` já recarrega a lista. Sem requisito de tempo real.
- [Passar só o número perde a unidade do `StockItem`] → A unidade já existe em `product.unit`; usa-se essa, evitando acoplar o form ao tipo `StockItem`.
