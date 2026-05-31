## Context

A perda já é fixada na compra (`do_purchase_work`), mas o `ProductForm` manteve um input editável de `expected_loss_rate` e cravou `loss_mode = "manual"` no payload, ignorando o toggle que o spec ainda descreve. Resultado: duas telas escrevem o mesmo campo, e o conceito de modo (manual/auto) não tem mais onde ser escolhido de forma coerente.

O negócio é uma loja de fotos (~80% produção, ~20% revenda). A perda real acontece nos **insumos comprados** — é por isso que o evento de compra é o lugar natural para registrá-la. Produtos vendáveis com ficha técnica derivam o CMV dos componentes (mudança `perda-de-insumo-na-compra`) e já entram com `expected_loss_rate = 0`; o campo de perda no `ProductForm` só aparece para produtos **sem** componentes (insumos e revenda), que são justamente os comprados.

Nenhuma coluna nova é necessária: `Products.loss_mode` (default `manual`) e `Purchase_Items.loss_rate` já existem.

## Goals / Non-Goals

**Goals:**
- A taxa de perda é editada em um único lugar: a compra.
- O modo manual/automático é uma escolha explícita por item de compra, persistida em `Products.loss_mode`.
- O formulário de produto exibe perda e modo em leitura, sem poder alterá-los.
- A compra é a única escritora de `expected_loss_rate`/`loss_mode` após a criação do produto.

**Non-Goals:**
- Refixar `cost_price` do insumo na compra (segue manual; era Open Question 1 da mudança anterior, adiada).
- Snapshot de `loss_mode` por compra em `Purchase_Items` (basta `loss_rate` para auditoria do valor; o modo vive no produto).
- Qualquer alteração no rollup de CMV pela ficha técnica.

## Decisions

### Perda somente leitura no produto (Opção A)

**What:** O input de perda do `ProductForm` é substituído por um display read-only de `expected_loss_rate` com rótulo do modo (`auto`/`manual`). O cálculo de CMV, margem e preço sugerido continua lendo esse valor persistido (0 em produto novo).

**Why:** Mantém a transparência de onde vem o piso de preço sem permitir uma segunda fonte de escrita. Descartou-se sumir com a perda do form inteiro (Opção B) porque deixaria o piso de preço sem explicação visível.

### Modo escolhido por item na compra (toggle explícito)

**What:** Cada item do `PurchaseForm` tem um seletor `Automático`/`Manual`. Em `auto`, a taxa sugerida (`get_product_loss_rate`) aparece em leitura e o item envia `loss_rate = null` + `loss_mode = "auto"`; o backend congela a taxa medida no momento da compra. Em `manual`, o input fica editável e o item envia `loss_rate = valor/100` + `loss_mode = "manual"`.

**Why:** Derivar o modo de "o operador mexeu no valor?" é frágil (retypar o mesmo número confunde). O toggle torna a intenção explícita e mapeia direto para `loss_mode`. Enviar `null` em `auto` reaproveita o fallback que `do_purchase_work` já tem (`loss_rate.unwrap_or_else(get_product_loss_rate_raw)`), preservando o snapshot no instante da compra.

### Compra como única escritora pós-criação

**What:** `update_product` deixa de receber e gravar `expected_loss_rate`/`loss_mode` (saem do `SET`). `create_product` deixa de recebê-los do form; o INSERT usa `expected_loss_rate = 0` e a coluna `loss_mode` cai no default `manual`.

**Why:** Com o form mandando um valor read-only carregado na abertura, um salvamento poderia sobrescrever uma compra mais recente (janela entre abrir e salvar). Tirando os campos do caminho de escrita do produto, a perda só anda pela compra — comportamento correto e sem corrida.

## Risks / Trade-offs

- **Produto novo nasce com perda 0.** Não dá mais para chutar uma perda inicial no cadastro; ela só aparece após a primeira compra do insumo. Coerente com o modelo "a perda é fenômeno do que se compra", e aceito na exploração.
- **Quebra de assinatura dos comandos.** `create_product`/`update_product` mudam de assinatura; os testes Rust existentes que inserem via SQL direto não dependem dos params, mas os `invoke` do front precisam ser ajustados juntos.
- **`loss_mode` vira read-only no produto.** Quem esperava editar o modo na tela de produto perde isso; é o objetivo da mudança.

## Migration Plan

Nenhuma migração de schema. As colunas já existem. A mudança é de comportamento de escrita (comandos Rust) e de UI (dois formulários). Produtos legados mantêm seus valores atuais de `expected_loss_rate`/`loss_mode`; a próxima compra de cada insumo passa a refixá-los pelo novo fluxo.
