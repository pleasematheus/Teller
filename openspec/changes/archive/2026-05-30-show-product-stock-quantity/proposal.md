## Why

Ao abrir um produto para edição, o operador não vê quanto resta em estoque daquele item e precisa sair para a tela de Estoque para conferir. Mostrar o saldo atual direto na tela do produto evita essa troca de contexto.

## What Changes

- Exibir, na tela de edição de produto (`ProductForm`), a **quantidade atual em estoque** do produto, em modo somente leitura.
- O saldo aparece apenas para produtos existentes (em edição); na criação de um produto novo ainda não há saldo a mostrar.
- Reutilizar o saldo já fornecido pelo backend (`list_stock` retorna `current_quantity` por produto). Nenhum comando Rust novo nem alteração de schema.
- Sem qualquer edição do saldo a partir desta tela: o ajuste de estoque continua exclusivo do módulo de Estoque.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `product-management`: acrescenta requisito de exibição somente-leitura da quantidade em estoque na tela de edição de produto.

## Impact

- Frontend: `src/components/products/ProductForm.tsx` (novo campo de leitura), `src/components/products/index.tsx` (carregar e repassar o saldo) e possivelmente `src/types/stock.ts` (reuso do tipo `StockItem`).
- Backend: nenhum. `list_stock` já expõe `current_quantity` para todos os produtos.
- Sem novas dependências.
