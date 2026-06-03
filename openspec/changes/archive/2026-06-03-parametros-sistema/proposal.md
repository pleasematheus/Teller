## Why

O sistema não possui um lugar dedicado e editável para parâmetros globais de comportamento (ex.: permitir venda com estoque negativo sem avisos). Hoje `App_Meta` guarda apenas metadados de máquina (`db_version`, `app_version`) lidos pela ferramenta de atualização e não deve ser exposta à edição pelo usuário. Sem uma tabela própria, qualquer toggle global teria que conviver com esses metadados de migração, arriscando corromper o estado lido pelo atualizador.

## What Changes

- Nova tabela `System_Settings` (chave/valor) para parâmetros globais editáveis pelo usuário, isolada de `App_Meta`.
- Seed inicial no `init_schema`: `allow_negative_stock_sale = 'false'`.
- Novos comandos Tauri em `settings.rs` para ler e gravar parâmetros do sistema (UPSERT com `updated_at`).
- A venda passa a ler `allow_negative_stock_sale`: quando `'false'` (padrão) mantém o comportamento atual (avisa e prossegue); quando `'true'` prossegue sem emitir avisos de estoque insuficiente. A venda nunca é bloqueada.
- Nova seção/toggle em Configurações no frontend para alternar o parâmetro.
- **BREAKING**: incremento de `DB_VERSION` "2" -> "3". Bancos existentes precisam ser recriados (deletar antigo e criar novo).

## Capabilities

### New Capabilities
- `parametros-sistema`: tabela `System_Settings` chave/valor para parâmetros globais editáveis, comandos de leitura/escrita e o parâmetro `allow_negative_stock_sale`.

### Modified Capabilities
- `sales-management`: os cenários de estoque insuficiente passam a condicionar a emissão de avisos ao parâmetro `allow_negative_stock_sale`.

## Impact

- Schema: `src/db/schema.sql` (nova tabela + seed), `src-tauri/src/db.rs` (`DB_VERSION` 2 -> 3, seed do parâmetro no `init_schema`).
- Backend: `src-tauri/src/settings.rs` (comandos get/set), `src-tauri/src/sales.rs` (leitura da flag e condicional dos avisos), `src-tauri/src/lib.rs` (registro dos comandos).
- Frontend: seção de Configurações (toggle) e função pura em `src/lib/` com teste.
- Dados: bancos existentes em disco precisam ser recriados devido ao bump de schema.
