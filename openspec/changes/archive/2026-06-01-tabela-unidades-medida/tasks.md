## 1. Schema e versão do banco

- [x] 1.1 Adicionar tabela `Units_Of_Measure` (sequência + `id`, `name` UNIQUE NOT NULL, `abbreviation` NOT NULL, `active` DEFAULT TRUE, `updated_at`, `created_at`) em `src/db/schema.sql`, criada vazia (sem seed)
- [x] 1.2 Remover `CHECK (unit IN (...))` da coluna `Products.unit` em `schema.sql`, mantendo `NOT NULL DEFAULT 'unidade'`
- [x] 1.3 Incrementar `DB_VERSION` em `src-tauri/src/db.rs`
- [x] 1.4 Atualizar/ajustar testes de `init_schema` em `db.rs` que dependem da presença de tabelas, se necessário

## 2. Backend: comandos de unidade de medida (settings.rs)

- [x] 2.1 Adicionar struct `UnitOfMeasure { id, name, abbreviation, active }` com `#[derive(Serialize)]` em `src-tauri/src/settings.rs`
- [x] 2.2 Implementar `list_units_of_measure` (ordenado por `active DESC, name`)
- [x] 2.3 Implementar `create_unit_of_measure(name, abbreviation)` com `trim`, rejeição de nome/sigla vazios e nome duplicado
- [x] 2.4 Implementar `rename_unit_of_measure(id, name, abbreviation)` que atualiza a unidade e propaga `UPDATE "Products" SET unit = ? WHERE unit = ?` (nome antigo → novo) na mesma transação
- [x] 2.5 Implementar `update_unit_of_measure(id, active)` para alternar ativo
- [x] 2.6 Implementar `delete_unit_of_measure(id)` com guarda: contar `Products WHERE unit = name`; bloquear com erro se > 0
- [x] 2.7 Registrar os 5 comandos em `tauri::generate_handler!` em `src-tauri/src/lib.rs`

## 3. Backend: validação no cadastro/edição de produto

- [x] 3.1 Em `src-tauri/src/products.rs`, validar `unit` no `create_product` contra `Units_Of_Measure` (existir e estar ativo), retornando erro visível quando inválido
- [x] 3.2 Validar `unit` no `update_product` contra `Units_Of_Measure` (existir), retornando erro quando inexistente

## 4. Backend: testes (Rust)

- [x] 4.1 Teste: `create_unit_of_measure` rejeita nome vazio e sigla vazia
- [x] 4.2 Teste: `create_unit_of_measure` insere com `active = true` e rejeita nome duplicado
- [x] 4.3 Teste: `rename_unit_of_measure` propaga novo nome para `Products.unit` dos produtos afetados
- [x] 4.4 Teste: `update_unit_of_measure` alterna `active`
- [x] 4.5 Teste: `delete_unit_of_measure` remove unidade sem uso e bloqueia quando há produto vinculado
- [x] 4.6 Teste: `create_product`/`update_product` rejeitam `unit` inexistente em `Units_Of_Measure`

## 5. Frontend: tipos e seção de Configurações

- [x] 5.1 Adicionar tipo `UnitOfMeasure { id: number; name: string; abbreviation: string; active: boolean }` (em `src/types/` ou na própria seção)
- [x] 5.2 Criar `src/components/configuracoes/UnidadesSection.tsx` espelhando `PagamentosSection` (lista, criar com nome+sigla, editar nome/sigla, alternar ativo, excluir com erro de uso)
- [x] 5.3 Adicionar entrada `unidades` em `NAV_ITEMS`, no tipo `Section` e no render de `ConfiguracoesModule.tsx`

## 6. Frontend: seletor dinâmico no ProductForm

- [x] 6.1 Em `src/components/products/ProductForm.tsx`, carregar unidades ativas via `invoke("list_units_of_measure")` (useEffect)
- [x] 6.2 Substituir as `<option>` fixas do `<select>` de unidade por opções dinâmicas exibindo nome e sigla
- [x] 6.3 Garantir que, em edição, a unidade atual do produto permaneça selecionada mesmo se desativada

## 7. Frontend: testes

- [x] 7.1 Se houver lógica pura extraída (ex.: formatação "nome (sigla)" ou filtragem de ativos), adicionar `*.test.ts` em `src/lib/` cobrindo-a

## 8. Verificação

- [x] 8.1 Rodar `cargo test` (subagente) e reportar resultado
- [x] 8.2 Rodar `bun run test` (subagente) e reportar resultado
- [x] 8.3 [MANUAL — testador humano] Recriar banco de dev: deletar o `.db` antigo e abrir o app para gerar o schema novo com a tabela `Units_Of_Measure`
- [x] 8.4 [MANUAL — testador humano] Em Configurações > Unidades de Medida: criar, renomear (verificar propagação em produto existente), desativar e tentar excluir uma unidade em uso (deve bloquear)
- [x] 8.5 [MANUAL — testador humano] No formulário de produto, confirmar que o seletor lista as unidades ativas com sigla e salva corretamente