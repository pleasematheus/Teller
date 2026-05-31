## Why

A taxa de perda (`expected_loss_rate`) ainda é editável em dois lugares: no formulário de produto (`ProductForm.tsx:305-319`) e no de compra. Desde a mudança `perda-de-insumo-na-compra`, a compra é o evento que **fixa** a perda — `do_purchase_work` (`stock.rs:101-122`) já congela o valor em `Products.expected_loss_rate` e em `Purchase_Items.loss_rate`. Mas o campo digitável continuou no produto, criando duas fontes de verdade para o mesmo número.

O modo manual/automático (`loss_mode`) também ficou órfão: o `ProductForm` já abandonou o toggle e crava `"manual"` no payload (`ProductForm.tsx:197`), enquanto o spec ainda descreve um seletor que não existe. A mudança `perda-de-insumo-na-compra` deixou isso explícito como Open Question 4 ("destino de `loss_mode`"). Esta mudança fecha esse ponto: o modo passa a ser escolhido na compra.

## What Changes

- **Produto deixa de editar a perda.** No `ProductForm`, o input de taxa de perda vira exibição **somente leitura** de `expected_loss_rate`, com rótulo do modo (`auto`/`manual`). O cálculo de CMV, margem e preço sugerido continua usando o valor persistido.
- **Compra é a única fonte da perda e do modo.** O `PurchaseForm` ganha um seletor **Automático/Manual por item**: em automático a taxa sugerida aparece em leitura; em manual o operador informa o valor. O modo escolhido grava em `Products.loss_mode`.
- **Compra vira a única escritora de `expected_loss_rate` e `loss_mode` após a criação.** `update_product` para de tocar nos dois campos; `create_product` os defaulta (produto novo nasce com perda 0 e modo `manual`). Isso elimina o caso de um salvamento de produto sobrescrever, com um valor de form velho, a perda fixada por uma compra mais recente.

## Impact

- Affected specs: product-management, stock-management
- Affected code:
  - `src/components/products/ProductForm.tsx` — perda read-only; remoção do input e do envio de `expectedLossRate`/`lossMode`
  - `src/components/purchases/PurchaseForm.tsx` — toggle auto/manual por item; envio de `loss_mode`
  - `src-tauri/src/stock.rs` — `PurchaseItemInput.loss_mode`; `do_purchase_work` grava `Products.loss_mode`
  - `src-tauri/src/products.rs` — `create_product`/`update_product` deixam de receber `expected_loss_rate`/`loss_mode` editáveis
  - `src-tauri/src/lib.rs` — assinaturas dos comandos `create_product`/`update_product`
  - `src/types/product.ts` — tipo do item de compra com `loss_mode`
- Sem migração de banco: `Products.loss_mode` e `Purchase_Items.loss_rate` já existem das mudanças `persist-loss-rate-mode` e `perda-de-insumo-na-compra`.
