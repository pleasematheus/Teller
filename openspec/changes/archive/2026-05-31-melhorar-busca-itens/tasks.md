## 1. Dependência e wrapper de UI

- [x] 1.1 Adicionar `cmdk` ao `package.json` e instalar (`bun add cmdk`)
- [x] 1.2 Criar `src/components/ui/command.tsx` (wrapper shadcn sobre `cmdk`): `Command`, `CommandInput`, `CommandList`, `CommandEmpty`, `CommandGroup`, `CommandItem`, com classes Tailwind dos tokens `teller-*`

## 2. Lógica de busca pura

- [x] 2.1 Criar `src/lib/productSearch.ts` com `normalize(s)` (lowercase + remoção de diacríticos via NFD), `matchesProduct(product, query)` e `rankProducts(products, query)` (prefixo no name > substring no name > match na description)
- [x] 2.2 Criar `src/lib/productSearch.test.ts` (Vitest) cobrindo: insensível a acento, match na descrição, ranqueamento prefixo > substring, query vazia
- [x] 2.3 Rodar `bun run test` e garantir que os testes de busca passam

## 3. Componente ProductCommand compartilhado

- [x] 3.1 Criar `src/components/ui/ProductCommand.tsx` com props `{ products, onSelect, selectedIds? }`, usando `Command` (cmdk) com `filter` apoiado em `rankProducts`, `CommandEmpty` para estado vazio e ícone de busca via `@hugeicons/react`
- [x] 3.2 Marcar na lista os itens cujo id está em `selectedIds` (indicação visual de "já no pedido")

## 4. Integração na venda (SaleForm)

- [x] 4.1 Substituir o dropdown inline de produto em `SaleForm.tsx` pelo `ProductCommand` ancorado no `Popover` existente, removendo `search`, `searchOpen`, `searchRef` e o `useEffect` de click-fora
- [x] 4.2 Adicionar campo "Qtd" (default 1) ao lado da busca; `onSelect` adiciona a quantidade informada ao carrinho (incrementa se já existe) e reseta o campo para 1
- [x] 4.3 Passar os ids do carrinho como `selectedIds` para marcar itens já adicionados

## 5. Integração na compra (PurchaseForm)

- [x] 5.1 Substituir o `<select>` nativo de produto por linha em `PurchaseForm.tsx` por um `Popover` com `PopoverTrigger` (botão mostrando o produto escolhido ou "Selecione...") e `PopoverContent` = `ProductCommand`
- [x] 5.2 No `onSelect`, definir `productId` da linha e disparar `get_product_loss_rate` preservando o comportamento atual (preenchimento de `lossRatePct`)

## 6. Verificação

- [x] 6.1 Rodar `bun run test` e `tsc` (build) sem erros
- [ ] 6.2 [Teste manual - tester humano] Venda: buscar com e sem acento, navegar por teclado (setas/Enter/Esc), estado vazio, adicionar com quantidade, item já no carrinho marcado e incrementado
- [ ] 6.3 [Teste manual - tester humano] Compra: abrir o seletor por linha, escolher produto, confirmar que o nome aparece e que a perda sugerida é preenchida via `get_product_loss_rate`
