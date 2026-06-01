## Why

As unidades de medida de produtos hoje são uma lista fixa embutida no banco (CHECK constraint em `Products.unit`) e duplicada como `<option>` estáticas no formulário de produto. Adicionar, renomear ou remover uma unidade exige editar `schema.sql` e o código do front, com deploy. Não há sigla associada a cada unidade. Uma tabela própria, gerenciável por uma seção em Configurações, remove essa rigidez e adiciona a sigla.

## What Changes

- Nova tabela `Units_Of_Measure` (id, nome único, sigla, ativo, timestamps) no schema, criada vazia. O usuário cadastra as unidades pela seção de Configurações.
- Novos comandos Tauri: `list_units_of_measure`, `create_unit_of_measure`, `rename_unit_of_measure`, `update_unit_of_measure` (ativo) e `delete_unit_of_measure` (bloqueado quando a unidade está em uso por algum produto), espelhando o padrão de `Payment_Methods`.
- Nova seção "Unidades de Medida" em Configurações, com listagem, criação, edição (nome + sigla + ativo) e exclusão, espelhando `PagamentosSection`.
- `Products.unit` deixa de ter CHECK constraint fixo; o valor passa a ser validado contra `Units_Of_Measure` no cadastro/edição. **BREAKING** (schema): remoção do CHECK exige recriar o banco via ferramenta de atualização dedicada; `db_version` é incrementado.
- O `<select>` de unidade no `ProductForm` passa a ser populado dinamicamente por `list_units_of_measure` (apenas unidades ativas), exibindo nome e sigla, no lugar das `<option>` fixas.

## Capabilities

### New Capabilities
- `configuracoes-unidades-medida`: seção de Configurações e comandos CRUD para gerenciar a tabela de unidades de medida (nome, sigla, ativo), com guarda de exclusão para unidades em uso.

### Modified Capabilities
- `product-management`: a validação de `unit` no cadastro/edição passa a referenciar `Units_Of_Measure` em vez de um conjunto fixo; o seletor de unidade do formulário é populado dinamicamente.
- `db-schema-initialization`: o schema passa a incluir a tabela `Units_Of_Measure` (criada vazia); `Products.unit` perde o CHECK constraint fixo e `db_version` é incrementado.

## Impact

- Schema: `src/db/schema.sql` (nova tabela, remoção do CHECK em `Products.unit`), `src-tauri/src/db.rs` (`DB_VERSION`).
- Backend: novo módulo/comandos em `src-tauri/src/settings.rs` (ou `units.rs`), registro em `src-tauri/src/lib.rs`, validação em `src-tauri/src/products.rs`.
- Frontend: `src/components/configuracoes/ConfiguracoesModule.tsx` (nova entrada de navegação), nova `UnidadesSection.tsx`, `src/components/products/ProductForm.tsx` (select dinâmico).
- Banco existente: requer recriação pela ferramenta de atualização dedicada (sem migração automática na inicialização).
