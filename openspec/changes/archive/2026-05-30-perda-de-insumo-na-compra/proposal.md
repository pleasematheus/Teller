## Why

Hoje a taxa de perda é um campo por produto. No modo automático ela é calculada como `Σperda / Σ(todas as entradas)` sobre todo o histórico (`get_product_loss_rate`, `products.rs:143`). Dois problemas tornam o número pouco confiável:

1. **A taxa flutua sozinha.** O denominador inclui unidades que ainda estão paradas em estoque e nunca tiveram chance de virar perda. Estoque novo e cheio faz a perda parecer baixa; conforme o estoque envelhece, a taxa sobe. Nunca é um valor fixo no qual confiar para precificar.
2. **Dá 0 para os produtos produzidos.** Produtos vendáveis (fotos) são produzidos sob demanda no fechamento da venda (`close_sale` consome os componentes, `sales.rs:301`) e **não têm entradas de estoque próprias**. Logo o cálculo automático retorna 0 para eles. Como ~80% do negócio é produção, a perda real (papel e bobina desperdiçados na troca de rolo e na impressão errada) acontece nos **insumos comprados**, não no produto final — e nunca entra na precificação da foto.

A perda, nesse negócio, é um fenômeno do que se **compra**. O insumo é comprado, fica parado, e se perde. A foto não estraga em estoque porque não existe foto em estoque. Mover a medição da perda para o insumo comprado, fixada na compra, e fazer o produto vendável herdar pela ficha técnica gera preços confiáveis para os dois lados do negócio (produção e revenda) com uma regra só.

## What Changes

- **Nova fórmula de perda por insumo**, sobre unidades resolvidas e janela de 12 meses:
  `loss = Σperda(12m) / (Σvenda(12m) + Σperda(12m))`.
  Unidades ainda em estoque não entram em nenhum lado da fração, então a taxa para de flutuar. A janela de 12 meses segue a mesma convenção de `get_realized_margin` (`products.rs:187`).
- **Perda fixada na compra.** `create_purchase` passa a sugerir a taxa medida por insumo, permitir override manual por item, e congelar o valor em `Products.expected_loss_rate`, com snapshot em `Purchase_Items.loss_rate` para auditoria. O botão "Calcular" flutuante do formulário de produto é removido — a compra passa a ser o evento que fixa a perda.
- **Rollup pela ficha técnica.** O CMV de um produto vendável passa a ser **derivado dos componentes**: `CMV = Σ [ custo_insumo / (1 − loss_insumo) ] × qtd`, em vez de usar `cost_price` e `expected_loss_rate` próprios digitados. O custo do produto vendável deixa de ser campo manual e passa a ser calculado a partir da ficha técnica. Produtos sem componentes (revenda) continuam usando a própria perda e o próprio custo.

## Impact

- **Affected specs:** product-management, stock-management
- **Affected code:**
  - `src-tauri/src/products.rs` — `get_product_loss_rate` (nova fórmula); nova função de rollup de CMV
  - `src-tauri/src/stock.rs` — `create_purchase` (sugestão + snapshot de perda)
  - `src/db/schema.sql` + `src-tauri/src/db.rs` — coluna `Purchase_Items.loss_rate` (migração idempotente)
  - `src/lib/pricing.ts` — função de rollup do CMV pela ficha técnica
  - `src/components/products/ProductForm.tsx` — custo derivado para vendáveis, remoção do botão auto
  - `src/components/purchases/PurchaseForm.tsx` — campo de perda sugerida/editável por item
  - `src/types/product.ts`
