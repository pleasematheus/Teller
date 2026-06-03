## 1. Schema e versionamento

- [x] 1.1 Adicionar tabela `System_Settings` (key TEXT PK, value TEXT NOT NULL, updated_at TIMESTAMP NOT NULL) em `src/db/schema.sql`
- [x] 1.2 Em `src-tauri/src/db.rs`, no `init_schema`, inserir seed `allow_negative_stock_sale = 'false'` em `System_Settings` (com `updated_at = CURRENT_TIMESTAMP`) após `execute_batch(SCHEMA)`
- [x] 1.3 Incrementar `DB_VERSION` de "2" para "3" em `src-tauri/src/db.rs`
- [x] 1.4 Teste Rust em `db.rs`: banco novo cria `System_Settings` e seed `allow_negative_stock_sale = 'false'` presente
- [x] 1.5 Teste Rust em `db.rs`: `App_Meta` continua apenas com `db_version` e `app_version` após `init_schema`

## 2. Backend - comandos de parâmetros

- [x] 2.1 Em `src-tauri/src/settings.rs`, função pura/auxiliar para resolver padrão por chave (ex.: `allow_negative_stock_sale` -> `'false'`) e normalizar booleanos para `'true'`/`'false'`
- [x] 2.2 Implementar command `get_system_setting(key) -> String` lendo `System_Settings`, retornando padrão da chave quando ausente
- [x] 2.3 Implementar command `set_system_setting(key, value)` com UPSERT em `System_Settings` atualizando `updated_at`
- [x] 2.4 Registrar os comandos no `invoke_handler` em `src-tauri/src/lib.rs`
- [x] 2.5 Teste Rust em `settings.rs`: get de chave ausente retorna padrão
- [x] 2.6 Teste Rust em `settings.rs`: set insere quando ausente e atualiza sem duplicar quando existente
- [x] 2.7 Teste Rust em `settings.rs`: set de booleano normaliza valor para `'true'`/`'false'`

## 3. Backend - gate de estoque negativo na venda

- [x] 3.1 Em `src-tauri/src/sales.rs` `close_sale`, ler `allow_negative_stock_sale` uma vez no início e converter para `bool`
- [x] 3.2 Condicionar os `warnings.push(...)` de estoque insuficiente (ramo de insumo e ramo de produto de revenda) a `!allow_negative`; manter a baixa via `Stock_Movements` em ambos os casos
- [x] 3.3 Teste Rust em `sales.rs`: com flag `'false'` (padrão), estoque insuficiente de produto de revenda retorna aviso e prossegue
- [x] 3.4 Teste Rust em `sales.rs`: com flag `'false'`, estoque insuficiente de insumo retorna aviso e prossegue
- [x] 3.5 Teste Rust em `sales.rs`: com flag `'true'`, estoque insuficiente prossegue sem avisos e baixa o estoque (fica negativo)

## 4. Frontend

- [x] 4.1 Função pura em `src/lib/` para parsear o valor do parâmetro (TEXT -> bool) com fallback ao padrão
- [x] 4.2 Teste Vitest `*.test.ts` da função de parse (valores `'true'`, `'false'`, ausente/ inválido)
- [x] 4.3 Adicionar seção/toggle "Permitir venda com estoque negativo" na tela de Configurações, invocando `get_system_setting`/`set_system_setting`
- [x] 4.4 Refletir o estado salvo ao abrir a tela e persistir alterações via `set_system_setting`

## 5. Migração de banco

- [x] 5.1 [MANUAL] Deletar o banco DuckDB existente em disco (versão 2) e iniciar o app para recriar na versão 3 (delegar a teste humano)

## 6. Validação

- [x] 6.1 Rodar `cargo test` (subagente) e reportar resultados [compilação OK; execução bloqueada por Windows App Control - rodar manualmente]
- [x] 6.2 Rodar `bun run test` (subagente) e reportar resultados [66/66 passed]
- [x] 6.3 [MANUAL] Verificar no app: toggle desligado avisa em venda sem estoque; ligado prossegue sem aviso (delegar a teste humano)
