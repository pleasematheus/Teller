## Why

O switch "Bloquear vendas com estoque negativo" hoje vive na aba "Sistema" das Configuracoes (`SistemaSection.tsx`). Esse parametro e uma regra de vendas, nao de sistema. A aba "Sistema" vai passar a hospedar configuracoes de comportamento do app (ex.: minimizar para bandeja, em mudanca separada). Separar o switch numa secao propria "Vendas" deixa a organizacao coerente e abre espaco para Sistema.

## What Changes

- Criar nova secao "Vendas" no sidebar de Configuracoes, posicionada ANTES de "Sistema".
- Mover o switch "Bloquear vendas com estoque negativo" de `SistemaSection` para uma nova `VendasSection`.
- A aba "Sistema" fica sem controles ate a mudanca de bandeja (mantem cabecalho/descricao como placeholder).
- Sem mudanca de backend: a leitura/gravacao do parametro `allow_negative_stock_sale` permanece identica.

## Capabilities

### New Capabilities
- `configuracoes-vendas`: secao "Vendas" das Configuracoes que agrupa parametros de comportamento de vendas, comecando pelo bloqueio de venda com estoque negativo.

### Modified Capabilities

## Impact

- `src/components/configuracoes/ConfiguracoesModule.tsx`: tipo `Section`, `NAV_ITEMS` (novo item "vendas" antes de "sistema"), render condicional.
- `src/components/configuracoes/VendasSection.tsx`: novo arquivo com o switch movido.
- `src/components/configuracoes/SistemaSection.tsx`: remover o switch; manter cabecalho como placeholder.
- Sem mudanca em Rust, schema ou parametros backend.
