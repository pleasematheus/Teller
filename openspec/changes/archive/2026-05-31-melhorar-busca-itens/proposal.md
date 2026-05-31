## Why

A seleção de produtos hoje não segue nenhum padrão de combobox. Na venda (`SaleForm`) a lista de resultados é um `<div>` renderizado no fluxo normal do documento, que empurra o conteúdo e aparece desalinhado; não há navegação por teclado nem estado vazio. Na compra (`PurchaseForm`) é um `<select>` nativo do sistema operacional, fora do design system e sem busca. Para um fluxo operacional de balcão isso é lento e visualmente inconsistente.

## What Changes

- Novo componente de UI compartilhado `ProductCommand` (combobox baseado em cmdk + `Command` do shadcn, ancorado via Popover já existente), com input de busca, lista filtrada, estado vazio e navegação por teclado nativa. Emite `onSelect(product)`.
- Lógica de busca pura em `src/lib/productSearch.ts`: normaliza acentos, casa por `name` + `description`, ranqueia prefixo acima de substring. Usada como `filter` do cmdk e coberta por teste Vitest.
- Venda (`SaleForm`): substitui o dropdown inline pelo `ProductCommand`, com campo de quantidade ao lado (default 1, reseta após adicionar). Selecionar adiciona a quantidade informada ao carrinho; itens já no carrinho ficam marcados na lista.
- Compra (`PurchaseForm`): substitui o `<select>` nativo por um botão-trigger por linha que abre o `ProductCommand` e mostra o produto escolhido. Selecionar define o `productId` da linha e dispara `get_product_loss_rate` (comportamento atual preservado).
- Nova dependência: `cmdk` (aprovada). Novo wrapper `src/components/ui/command.tsx`. Ícones via `@hugeicons/react`.
- Sem alteração de banco: a busca usa apenas `name` e `description`, que já existem no modelo `Product`.

## Capabilities

### New Capabilities
- `product-search`: comportamento do combobox de seleção de produto compartilhado entre venda e compra (busca por nome/descrição insensível a acento, ranqueamento, navegação por teclado, estado vazio, seleção que emite o produto escolhido).

### Modified Capabilities
- `sales-management`: a adição de itens ao pedido passa a usar o combobox com quantidade no momento de adicionar e marcação de itens já presentes no carrinho.
- `stock-management`: a seleção de produto no registro de compra multi-item passa a usar o combobox no lugar do `<select>` nativo.

## Impact

- Código: `src/components/sales/SaleForm.tsx`, `src/components/purchases/PurchaseForm.tsx`, novos `src/components/ui/command.tsx`, `src/components/ui/ProductCommand.tsx`, `src/lib/productSearch.ts` (+ `.test.ts`).
- Dependências: adiciona `cmdk`.
- Banco de dados: nenhum (sem migração DuckDB).
- Comandos Tauri: nenhum novo; `get_product_loss_rate` continua sendo chamado pela compra.
