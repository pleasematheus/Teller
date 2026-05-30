## 1. Carregar o saldo no container de produtos

- [x] 1.1 Em `src/components/products/index.tsx`, importar `StockItem` de `@/types/stock` e adicionar estado `stockByProduct: Map<number, number>`.
- [x] 1.2 No `loadProducts` (ou no `useEffect` de mount), chamar `invoke<StockItem[]>("list_stock")` em paralelo e popular o map `product_id -> current_quantity`.
- [x] 1.3 Repassar `stockQuantity={formContent.kind === "edit" ? stockByProduct.get(formContent.product.id) : undefined}` ao `ProductForm`.

## 2. Exibir o saldo somente leitura no formulário

- [x] 2.1 Em `src/components/products/ProductForm.tsx`, adicionar `stockQuantity?: number` ao tipo `Props`.
- [x] 2.2 Adicionar helper de formatação de quantidade (inteiro sem casas, senão 2 casas), no padrão de `fmtQty` em `StockBalance.tsx`.
- [x] 2.3 Renderizar um campo de leitura (estilo do campo de custo derivado: `border`/`bg-teller-card`/`text-teller-muted`) com `quantidade + product.unit`, somente quando `isEdit` for verdadeiro.
- [x] 2.4 Garantir que o campo não tem input, botão de ajuste nem entra no `payload` de submissão.

## 3. Verificação

- [x] 3.1 Rodar `rtk tsc` e `rtk lint` e corrigir eventuais erros.
- [x] 3.2 Verificar manualmente: abrir produto com saldo mostra a quantidade; produto sem movimento mostra 0; criar produto novo não exibe o campo.
