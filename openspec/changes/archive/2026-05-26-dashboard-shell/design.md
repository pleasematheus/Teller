## Context

O `Dashboard.tsx` atual é uma casca com header fixo (logo, nome do usuário, botão Sair) e `<main>` vazio. Este design substitui esse shell por um sistema de navegação por abas com lançador central, mantendo toda a lógica de autenticação intacta no `App.tsx`.

## Goals / Non-Goals

**Goals:**
- Tab bar linha única com [🏠] permanente + abas por módulo
- Tela lançador com grade 3×2 de cards e saudação
- Comportamento left-click / right-click nos cards e nas abas
- Menus de contexto customizados (sem dependência externa)
- Estado de abas encapsulado no Dashboard (não vaza para App.tsx)

**Non-Goals:**
- Conteúdo funcional dos módulos (Estoque, Compras etc.)
- Dados sumários nos cards (planejado para futuro)
- Overflow de abas (scroll, dropdown) — edge case adiado
- Persistência de abas entre sessões

## Decisions

### Estado das abas: local no Dashboard

O `App.tsx` gerencia telas (screen state) e o usuário autenticado. O estado de abas é puramente UI interna ao Dashboard — não há razão para elevá-lo.

```
App.tsx        → screen: "dashboard", user: AuthUser
Dashboard.tsx  → tabs: Tab[], activeTab: string | null
```

`activeTab === null` significa que o lançador está ativo.

### Modelo de dados de aba

```typescript
type ModuleId = "estoque" | "compras" | "producao" | "vendas" | "produtos" | "configuracoes";

type Tab = {
  id: string;      // crypto.randomUUID() — permite múltiplas instâncias do mesmo módulo
  module: ModuleId;
};
```

`id` único por instância permite duas abas de "Estoque" coexistirem sem colisão.

### Comportamento de abertura

- Left-click no card: se já existe aba com esse `module` → ativa a primeira encontrada; senão → cria nova aba
- Right-click no card → menu: "Abrir" (mesmo que left-click) / "Abrir nova instância" (sempre cria nova)
- Right-click na aba → menu: "Duplicar" (cria nova aba do mesmo módulo) / "Fechar"

### Menu de contexto: shadcn ContextMenu (Radix UI)

Usar o componente `ContextMenu` do shadcn/ui em vez de implementação customizada. Lida com posicionamento na borda da tela, fechar ao clicar fora e acessibilidade de teclado. Wrapping: cada `ModuleCard` e cada aba na `TabBar` são envolvidos em `<ContextMenu>` com seus respectivos `<ContextMenuItem>`.

`Tooltip` (também instalado) será usado na `TabBar` para exibir o nome completo de abas quando truncadas.

### Estrutura de componentes

```
src/components/
  Dashboard.tsx          — root: gerencia tabs[], activeTab, renderiza shell
  dashboard/
    TabBar.tsx           — [🏠] + abas + usuário/logout
    Launcher.tsx         — saudação + grade 3×2
    ModuleCard.tsx       — card individual com onContextMenu
    ContextMenu.tsx      — menu posicionado reutilizável
```

### Ícones dos módulos

Usar `@hugeicons/react` + `@hugeicons/core-free-icons` (já disponíveis no projeto). Mapeamento:

| Módulo        | Ícone Hugeicons       |
|---------------|-----------------------|
| Estoque       | `WarehouseIcon`       |
| Compras       | `ShoppingCart01Icon`  |
| Produção      | `Factory01Icon`       |
| Vendas        | `Store01Icon`         |
| Produtos      | `Tag01Icon`           |
| Configurações | `Settings01Icon`      |
| Home          | `Home01Icon`          |
| Fechar aba    | `Cancel01Icon`        |

## Risks / Trade-offs

- [Menu de contexto custom] Clicar fora para fechar pode conflitar com outros listeners → Mitigation: usar `mousedown` com `stopPropagation` e cleanup no `useEffect`
- [Múltiplas instâncias] Labels de abas ficam idênticos (ex: "Estoque" e "Estoque") → Mitigation: aceito por ora; futuro pode adicionar índice ("Estoque 2")
- [Estado local] Abas se perdem ao fazer logout/login → esperado e desejado (sessão fresca)
