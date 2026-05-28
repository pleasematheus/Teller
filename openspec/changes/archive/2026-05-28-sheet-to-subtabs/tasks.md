## 1. Módulo Produtos

- [x] 1.1 Adicionar tipo `ProductsView` e `useState<ProductsView>` em `products/index.tsx`
- [x] 1.2 Substituir sub-tab strip do módulo Produtos com abas fixas ("Lista") e dinâmicas ("Novo produto", "Editar: [nome]")
- [x] 1.3 Remover props `open`, `onClose` de `ProductForm`; converter para renderização inline sem `Sheet/SheetContent/SheetHeader`
- [x] 1.4 Ajustar layout do `ProductForm` inline: `max-w-lg mx-auto px-6 py-6 overflow-y-auto`
- [x] 1.5 Passar `onSaved` e `onCancel` que setam view de volta para `{ kind: "list" }`
- [x] 1.6 Remover imports de `Sheet` de `ProductForm` e `products/index.tsx`

## 2. Módulo Estoque

- [x] 2.1 Estender tipo `StockView` em `StockModule.tsx` com `{ kind: "adjustment"; initialProductId?: number }`
- [x] 2.2 Adicionar sub-aba dinâmica "Ajuste ×" no strip existente de `StockModule`
- [x] 2.3 Remover props `open`, `onClose` de `StockAdjustmentForm`; converter para renderização inline sem `Sheet/SheetContent/SheetHeader`
- [x] 2.4 Ajustar layout do `StockAdjustmentForm` inline: `max-w-lg mx-auto px-6 py-6`
- [x] 2.5 Passar `onSaved` e `onCancel` que setam view de volta para `{ kind: "balance" }`
- [x] 2.6 Remover imports de `Sheet` de `StockAdjustmentForm` e `StockModule.tsx`

## 3. Módulo Vendas

- [x] 3.1 Estender tipo `SalesView` em `SalesModule.tsx` com `new-sale`, `view-sale`, `close-sale`
- [x] 3.2 Adicionar sub-abas dinâmicas no strip de `SalesModule`: "Novo pedido ×", "Pedido #N ×", "Fechar #N ×"
- [x] 3.3 Remover props `open`, `onClose` de `SaleForm`; converter para renderização inline sem `Sheet/SheetContent/SheetHeader`
- [x] 3.4 Ajustar layout do `SaleForm` inline: `max-w-lg mx-auto px-6 py-6 overflow-y-auto`
- [x] 3.5 Passar `onSaved` e `onCancel` que setam view de volta para `{ kind: "open" }`
- [x] 3.6 Renomear `SaleCloseSheet.tsx` → `SaleCloseForm.tsx` e remover wrapper `Sheet/SheetContent/SheetHeader`
- [x] 3.7 Ajustar layout do `SaleCloseForm` inline: `max-w-lg mx-auto px-6 py-6`
- [x] 3.8 Atualizar import em `SalesModule.tsx` para `SaleCloseForm`
- [x] 3.9 Remover imports de `Sheet` de todos os arquivos do módulo Vendas

## 4. Limpeza

- [x] 4.1 Verificar se `src/components/ui/sheet.tsx` ainda é importado em algum arquivo
- [x] 4.2 Remover `src/components/ui/sheet.tsx` se não usado
- [x] 4.3 Rodar `bun run test` e verificar que todos os testes passam
