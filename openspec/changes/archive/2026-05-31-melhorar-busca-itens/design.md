## Context

A seleção de produto existe em dois lugares com implementações divergentes e ruins:

- `SaleForm.tsx` usa estado próprio (`search`, `searchOpen`, `searchRef`) e renderiza a lista de resultados como um `<div>` no fluxo normal do documento (linha ~168), com `useEffect` manual de click-fora e filtro `name.toLowerCase().includes(q)` limitado a 8 itens. Sem teclado, sem estado vazio, e a lista empurra o conteúdo.
- `PurchaseForm.tsx` usa um `<select>` nativo por linha de item, estilizado por fora do design system e sem busca.

O projeto já tem `radix-ui` e `src/components/ui/popover.tsx` (Radix Popover, portalizado e ancorado). A dependência `cmdk` foi aprovada para esta mudança. Ícones vêm de `@hugeicons/react`. O modelo `Product` expõe `name` e `description` (sem `sku`/código), então a busca opera só nesses campos e não exige migração de banco.

## Goals / Non-Goals

**Goals:**
- Um único componente de combobox (`ProductCommand`) reutilizado em venda e compra, ancorado e flutuante via Popover, com teclado e estado vazio nativos do cmdk.
- Lógica de busca pura e testável em `src/lib/productSearch.ts` (acento, name+description, ranqueamento por prefixo), usada como `filter` do cmdk.
- Venda: adicionar com quantidade ao lado da busca e marcar itens já no carrinho.
- Compra: trigger por linha que substitui o `<select>` nativo preservando o disparo de `get_product_loss_rate`.

**Non-Goals:**
- Busca por código de barras / SKU (campo não existe no modelo; fora de escopo).
- Alteração de qualquer comando Tauri ou schema DuckDB.
- Mudança na lógica de carrinho/fechamento da venda ou no cálculo de compra além da troca do seletor.

## Decisions

**1. cmdk + Command do shadcn, ancorado no Popover existente.**
O cmdk traz filtro, navegação por teclado, foco e estado vazio (`CommandEmpty`) prontos e acessíveis. Envolvemos em `src/components/ui/command.tsx` (wrapper shadcn) e ancoramos via o `Popover`/`PopoverContent` que já existe. Alternativa considerada: construir tudo sobre o Popover à mão (sem dep). Rejeitada porque reimplementaria teclado e acessibilidade que o cmdk já entrega; a dep foi aprovada.

**2. Import do `command.tsx`.**
O `command.tsx` importa diretamente de `cmdk` (padrão shadcn), diferente do `popover.tsx` que usa o pacote unificado `radix-ui`. Motivo: o cmdk não faz parte do `radix-ui`; é um pacote próprio. O wrapper aplica as classes Tailwind do design system (tokens `teller-*`).

**3. `ProductCommand` é UI pura; lógica fica nos forms.**
Assinatura aproximada: `ProductCommand({ products, onSelect, selectedIds? })`. Ele só busca e emite `onSelect(product)`. `selectedIds` (opcional) marca itens já escolhidos. A venda passa os ids do carrinho; a compra não passa. Isso mantém o componente sem estado de negócio e testável isoladamente.

**4. Dois invólucros, um miolo.**
- Venda: `ProductCommand` sempre visível (input fixo) + campo "Qtd" ao lado (default 1). `onSelect` adiciona `qty` ao carrinho (incrementa se já existe) e reseta o campo para 1.
- Compra: um `Popover` com `PopoverTrigger` = botão por linha mostrando o produto escolhido (ou "Selecione..."), `PopoverContent` = `ProductCommand`. `onSelect` define `productId` da linha e dispara `get_product_loss_rate` (mesma lógica de `PurchaseForm.tsx:163-168`).

**5. Busca pura em `src/lib/productSearch.ts`.**
Funções: `normalize(s)` (lowercase + remove diacríticos via `normalize("NFD")` + regex de marcas), `matchesProduct(product, query)`, e `rankProducts(products, query)` (prefixo no `name` > substring no `name` > match na `description`). O cmdk recebe um `filter` que usa essa lógica para pontuar. Teste Vitest cobre acento, descrição e ordenação.

## Risks / Trade-offs

- [Nova dependência `cmdk` aumenta o bundle] → Aceito; é a base de combobox padrão do shadcn e foi aprovada. Sem migração de dados envolvida.
- [Filtro custom do cmdk pode conflitar com o filtro interno dele] → Mitigação: definir explicitamente a prop `filter` do `Command` apontando para `rankProducts`/score, evitando dupla filtragem.
- [Comportamento de `get_product_loss_rate` na compra pode regredir na troca do `<select>`] → Mitigação: preservar exatamente o callback atual e cobrir com task de teste manual da compra.
- [Diferença de import entre `command.tsx` (cmdk) e `popover.tsx` (radix-ui) pode confundir] → Documentado nesta decisão; é intencional.

## Migration Plan

Sem migração de banco. Passos: instalar `cmdk`, adicionar `command.tsx`, criar `productSearch.ts` + teste, criar `ProductCommand`, ligar em `SaleForm` e `PurchaseForm`, remover o estado/lista inline da venda e o `<select>` da compra. Rollback: reverter os arquivos de UI e remover a dependência; nada persistido muda.

## Open Questions

- Posição exata do campo "Qtd" na venda (ao lado da busca vs. acima): decisão de layout final fica para a implementação; o spec só exige que exista e resete para 1.
