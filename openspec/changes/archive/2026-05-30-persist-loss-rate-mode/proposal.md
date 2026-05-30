## Why

O modo da taxa de perda (Manual/Automático) só existe como estado local de UI no formulário de produto: ele sempre reinicia em "Manual" ao reabrir, porque não há campo persistido nem é enviado ao backend. O usuário que seleciona "Automático" perde a escolha na próxima edição.

## What Changes

- Adicionar `loss_mode` (`'manual' | 'auto'`) por produto, persistido em `Products`, com default `'manual'`.
- `create_product` e `update_product` passam a receber e gravar `loss_mode`.
- `list_products` passa a retornar `loss_mode`; o formulário restaura o toggle a partir do produto.
- O valor de `expected_loss_rate` continua só mudando por entrada manual ou ao acionar "Calcular"; nenhum recálculo automático na leitura. O modo permanece até a próxima edição.

## Impact

- Affected specs: product-management
- Affected code: `src/db/schema.sql`, `src-tauri/src/db.rs`, `src-tauri/src/products.rs`, `src-tauri/src/lib.rs`, `src/types/product.ts`, `src/components/products/ProductForm.tsx`
