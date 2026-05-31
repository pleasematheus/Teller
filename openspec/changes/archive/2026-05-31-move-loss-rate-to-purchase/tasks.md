## 1. Backend — compra grava perda e modo (Rust)

- [x] 1.1 Estender `PurchaseItemInput` com `loss_mode: Option<String>` em `stock.rs`
- [x] 1.2 Em `do_purchase_work`: validar `loss_mode` (default `manual`) com `validate_loss_mode`; manter `effective_loss = loss_rate ?? get_product_loss_rate_raw`; gravar `Products.loss_mode` junto com `expected_loss_rate`
- [x] 1.3 Teste: item em modo `auto` (loss_rate=None) congela a taxa sugerida e grava `loss_mode='auto'`
- [x] 1.4 Teste: item em modo `manual` com override grava o valor e `loss_mode='manual'`
- [x] 1.5 Teste: `loss_mode` inválido é rejeitado

## 2. Backend — produto deixa de escrever perda (Rust)

- [x] 2.1 Remover `expected_loss_rate` e `loss_mode` dos parâmetros de `update_product` e do `SET` da UPDATE
- [x] 2.2 Remover `expected_loss_rate`/`loss_mode` dos parâmetros de `create_product`; INSERT grava `expected_loss_rate = 0.0` e omite `loss_mode` (cai no default `manual`); ajustar a chamada de `validate_product`
- [x] 2.3 Atualizar as assinaturas/registros dos comandos em `lib.rs` (sem mudança: `generate_handler!` registra só por nome; a macro deriva os parâmetros)
- [x] 2.4 Teste: `update_product` não altera `expected_loss_rate` nem `loss_mode` de um produto cujo valor foi fixado por compra
- [x] 2.5 Teste: `create_product` cria com `expected_loss_rate = 0` e `loss_mode = 'manual'`

## 3. Frontend — produto (React)

- [x] 3.1 No `ProductForm.tsx`, remover o `<Input>` de taxa de perda e o estado editável (`lossRatePct`); exibir `expected_loss_rate` em leitura com rótulo do modo (`auto`/`manual`)
- [x] 3.2 Parar de enviar `expectedLossRate`/`lossMode` nos payloads de `create_product`/`update_product`; CMV/margem/preço sugerido passam a usar `product.expected_loss_rate` (0 na criação)
- [x] 3.3 Ajustar os `invoke` e os tipos para a nova assinatura dos comandos

## 4. Frontend — compra (React)

- [x] 4.1 No `PurchaseForm.tsx`, adicionar `lossMode: "auto" | "manual"` ao `ItemState` e um seletor por item
- [x] 4.2 Modo `auto`: exibe a taxa sugerida (`get_product_loss_rate`) em leitura e envia `loss_rate = null`; modo `manual`: input editável, envia `loss_rate = valor/100`
- [x] 4.3 Incluir `loss_mode` no payload de `create_purchase`

## 5. Tipos (TS)

- [x] 5.1 Atualizar `src/types/product.ts` (tipo do item de compra com `loss_mode`); confirmar que `pricing.ts`/`pricing.test.ts` não precisam de ajuste (CMV continua recebendo `expected_loss_rate` como argumento)

## 6. Verificação

- [x] 6.1 `rtk cargo test` (117 passou) e `rtk bun run test` (Vitest, 47 passou) verdes; `tsc --noEmit` sem novos erros nos arquivos alterados
- [x] 6.2 [MANUAL — tester humano] Abrir produto sem componentes para edição: taxa de perda aparece somente leitura com o modo; salvar não altera a perda
- [x] 6.3 [MANUAL — tester humano] Comprar insumo em `auto`: perda congela na taxa medida; em `manual`: congela no valor digitado; o `ProductForm` reflete o novo valor e modo

## 7. Correção — perda anotada não era medida (Rust)

Descoberto no teste 6.3: uma perda registrada com observação é gravada como `motive = "perda - <nota>"` (concatenação em `create_stock_adjustment`), e a medição filtrava `motive = 'perda'` exato, ignorando essas perdas e devolvendo 0% no modo `auto`.

- [x] 7.1 Em `get_product_loss_rate_raw`, extrair o motivo-base com `split_part(motive, ' - ', 1)` nos dois lados da fração (numerador e filtro), sem alterar o schema
- [x] 7.2 Teste: perda com motivo anotado (`'perda - <nota>'`) é contabilizada na taxa
