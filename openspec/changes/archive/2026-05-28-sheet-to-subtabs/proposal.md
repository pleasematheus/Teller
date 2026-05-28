## Why

Sheets (gavetas laterais) ocupam uma área pequena em desktop, forçam o usuário a trabalhar num painel estreito sobreposto ao conteúdo e introduzem uma camada de interação (abrir/fechar overlay) desnecessária numa app Tauri onde há espaço de tela completo disponível. Sub-abas dinâmicas dentro de cada módulo aproveitam o espaço inteiro e mantêm a consistência com o padrão de navegação já existente no app.

## What Changes

- Todos os 4 componentes Sheet são convertidos para renderização inline (sem `Sheet`/`SheetContent`/`SheetHeader`):
  - `ProductForm` → renderizado como sub-aba "Novo produto" ou "Editar: [nome]"
  - `StockAdjustmentForm` → renderizado como sub-aba "Ajuste"
  - `SaleForm` → renderizado como sub-aba "Novo pedido" ou "Pedido #N"
  - `SaleCloseSheet` → renderizado como sub-aba "Fechar #N"
- Cada módulo ganha um view-state discriminado que controla qual sub-aba está ativa
- Sub-abas fixas (Lista, Saldo, Em aberto, Histórico) não têm botão de fechar
- Sub-abas dinâmicas aparecem ao acionar a ação e somem ao salvar/cancelar, com perda silenciosa de estado não salvo
- `products/index.tsx` recebe sub-abas; hoje não tem nenhuma
- `StockModule` e `SalesModule` já têm sub-abas fixas — ganham as dinâmicas

## Capabilities

### New Capabilities
- `module-sub-navigation`: Padrão de navegação intra-módulo com view-state discriminado e sub-tab strip reutilizável. Cobre o componente `SubTabBar`, o hook `useModuleView`, e a integração nos três módulos.

### Modified Capabilities

## Impact

- `src/components/products/index.tsx` — adiciona sub-abas e view-state
- `src/components/products/ProductForm.tsx` — remove wrapper Sheet
- `src/components/stock/StockModule.tsx` — estende view-state com `adjustment`
- `src/components/stock/StockAdjustmentForm.tsx` — remove wrapper Sheet
- `src/components/sales/SalesModule.tsx` — estende view-state com `new-sale`, `view-sale`, `close-sale`
- `src/components/sales/SaleForm.tsx` — remove wrapper Sheet
- `src/components/sales/SaleCloseSheet.tsx` — remove wrapper Sheet, renomear para `SaleCloseForm.tsx`
- Novo componente `src/components/ui/sub-tab-bar.tsx` (ou inline nos módulos)
- `src/components/ui/sheet.tsx` — pode ser removido se não usado em mais nenhum lugar
