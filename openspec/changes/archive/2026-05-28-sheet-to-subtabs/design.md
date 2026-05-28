## Context

O app usa um TabBar de módulos (Produtos, Vendas, Estoque…) no topo — padrão de janelas tipo IDE. Dentro de módulos, `SalesModule` e `StockModule` já têm sub-abas fixas com botão-style `border-b-2`. Formulários são abertos via `Sheet` (shadcn, gaveta lateral de 448px max). A mudança elimina os Sheets e expande o padrão de sub-abas existente para incluir abas dinâmicas.

## Goals / Non-Goals

**Goals:**
- Formulários ocupam a largura total do módulo
- UX consistente: mesmo padrão de sub-abas em todos os módulos
- Sem nova dependência externa — usar o padrão de tab já existente inline
- Perda silenciosa de estado não salvo ao navegar para outra aba (paridade com fechar Sheet hoje)

**Non-Goals:**
- Aviso de "alterações não salvas" ao trocar de aba
- Múltiplos formulários abertos simultaneamente no mesmo módulo
- Animações de transição entre abas
- Mudanças no backend / comandos Tauri

## Decisions

### D1 — View-state discriminado por módulo (sem roteador)

Cada módulo mantém um `useState` local com tipo union:

```ts
// Produtos
type ProductsView =
  | { kind: "list" }
  | { kind: "new" }
  | { kind: "edit"; product: Product }

// Estoque
type StockView =
  | { kind: "balance" }
  | { kind: "movements" }
  | { kind: "adjustment"; initialProductId?: number }

// Vendas
type SalesView =
  | { kind: "open" }
  | { kind: "history" }
  | { kind: "new-sale" }
  | { kind: "view-sale"; sale: Sale }
  | { kind: "close-sale"; sale: Sale }
```

**Alternativa descartada**: roteador externo (TanStack Router, React Router). Overkill para um app sem URL bar; o padrão de state local já é usado em `SalesModule`.

### D2 — Sub-tab strip inline nos módulos (sem componente compartilhado novo)

O strip de sub-abas é construído inline em cada módulo usando o mesmo padrão `border-b-2 -mb-px` já existente. Não extrai componente genérico.

**Rationale**: Os três módulos têm particularidades (abas fixas vs dinâmicas, labels com ID da venda). Abstrair prematuramente adicionaria complexidade sem reutilização real. Se o padrão se repetir em mais módulos futuros, extrai-se então.

### D3 — Formulários viram componentes inline (sem prop `open`)

`ProductForm`, `StockAdjustmentForm`, `SaleForm` e `SaleCloseSheet` perdem o wrapper `Sheet/SheetContent/SheetHeader` e a prop `open: boolean`. São renderizados condicionalmente pelo módulo pai via view-state. Resetam estado via `useEffect` dependendo das props que chegam (product, sale, etc.) — o mesmo trigger já usado hoje quando `open` muda.

**Alternativa descartada**: manter prop `open` e apenas trocar o wrapper por um `div`. Criaria dead code (prop sempre `true` quando renderizado) e confusão para quem ler o código depois.

### D4 — Label das abas dinâmicas

| View | Label da aba |
|------|-------------|
| `new` / `new-sale` | "Novo produto" / "Novo pedido" |
| `edit` | "Editar: [nome]" |
| `view-sale` | "Pedido #[id]" |
| `close-sale` | "Fechar #[id]" |
| `adjustment` | "Ajuste" |

Aba dinâmica sempre tem botão `×` que seta view para a aba fixa padrão do módulo (`list` / `balance` / `open`).

### D5 — Layout dos formulários inline

Sem `SheetContent` (max-w-md + overflow-y-auto), o formulário recebe:
- `max-w-lg mx-auto` para não esticar demais em telas largas
- `overflow-y-auto flex-1` no container do módulo para scroll correto
- Padding `px-6 py-6` (mesmos valores que hoje dentro do Sheet)

## Risks / Trade-offs

- **Formulário muito largo em telas grandes** → mitigado por `max-w-lg mx-auto` na D5
- **`sheet.tsx` torna-se dead code** → remover após a migração; verificar que nenhum outro componente o usa

## Migration Plan

1. Migrar módulo por módulo (Produtos → Estoque → Vendas) para isolar regressões
2. Cada módulo: adicionar view-state → adaptar sub-tab strip → converter formulário → remover imports Sheet
3. Ao final, remover `src/components/ui/sheet.tsx` se não usado
4. Rollback: cada módulo é independente; reverter um não afeta os outros
