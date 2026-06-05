# Spec: configuracoes-unidades-medida

## Purpose

Gerenciamento das unidades de medida disponíveis para produtos. O operador cadastra, edita, desativa e exclui unidades via seção de Configurações. As unidades cadastradas são a fonte de verdade usada pelo formulário de produto e pela validação do backend.

## Requirements

### Requirement: Listagem de unidades de medida

O sistema SHALL expor o comando `list_units_of_measure` que retorna todas as unidades cadastradas em `Units_Of_Measure` com os campos `id`, `name`, `abbreviation`, `active` e `integer`, ordenadas por `active` decrescente e depois por `name`.

#### Scenario: Lista com unidades existentes
- **WHEN** o comando `list_units_of_measure` é invocado e há unidades cadastradas
- **THEN** o sistema retorna todos os registros com `id`, `name`, `abbreviation`, `active` e `integer`, com as ativas primeiro e ordenadas por nome

#### Scenario: Lista vazia
- **WHEN** o comando `list_units_of_measure` é invocado e não há unidades
- **THEN** o sistema retorna uma lista vazia sem erro

### Requirement: Cadastro de unidade de medida

O sistema SHALL expor o comando `create_unit_of_measure` que insere uma nova unidade com `name` e `abbreviation` não-vazios (após `trim`), `active = true` e o flag `integer` informado pelo chamador. O sistema SHALL rejeitar nome ou sigla vazios e nome duplicado.

#### Scenario: Cadastro válido inteiro
- **WHEN** o usuário submete `name` e `abbreviation` não-vazios, `integer = true` e o nome ainda não existe
- **THEN** o sistema insere a unidade com `active = true` e `integer = true` e retorna sucesso

#### Scenario: Cadastro válido decimal
- **WHEN** o usuário submete `name` e `abbreviation` não-vazios e `integer = false`
- **THEN** o sistema insere a unidade com `active = true` e `integer = false` e retorna sucesso

#### Scenario: Nome vazio
- **WHEN** o usuário submete `name` em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Sigla vazia
- **WHEN** o usuário submete `abbreviation` em branco
- **THEN** o sistema retorna erro de validação sem criar o registro

#### Scenario: Nome duplicado
- **WHEN** o usuário submete um `name` que já existe em `Units_Of_Measure`
- **THEN** o sistema retorna erro sem criar um registro duplicado

### Requirement: Edição de unidade de medida

O sistema SHALL expor o comando `rename_unit_of_measure` que atualiza `name`, `abbreviation` e `integer` de uma unidade existente. Ao alterar o `name`, o sistema SHALL propagar o novo nome para `Products.unit` de todos os produtos que usavam o nome anterior, na mesma transação. O sistema SHALL expor o comando `update_unit_of_measure` que altera apenas o flag `active`.

#### Scenario: Renomear propaga para produtos
- **WHEN** o usuário renomeia uma unidade de `nome_antigo` para `nome_novo`
- **THEN** o sistema atualiza `Units_Of_Measure.name` e executa `UPDATE "Products" SET unit = 'nome_novo' WHERE unit = 'nome_antigo'` na mesma transação

#### Scenario: Editar sigla não afeta produtos
- **WHEN** o usuário altera apenas a `abbreviation` de uma unidade
- **THEN** o sistema atualiza a sigla sem alterar nenhum registro de `Products`

#### Scenario: Alternar inteiro
- **WHEN** o usuário salva a edição de uma unidade alterando `integer`
- **THEN** o sistema grava o novo valor de `integer` em `Units_Of_Measure` sem alterar nenhum registro de `Products`

#### Scenario: Desativar unidade
- **WHEN** o usuário define `active = false` para uma unidade
- **THEN** a unidade deixa de aparecer no seletor de unidades do formulário de produto, sem afetar produtos que já a utilizam

#### Scenario: Unidade inexistente
- **WHEN** `rename_unit_of_measure` ou `update_unit_of_measure` é invocado com `id` que não existe
- **THEN** o sistema retorna erro sem alterar nenhum registro

### Requirement: Exclusão de unidade de medida com guarda de uso

O sistema SHALL expor o comando `delete_unit_of_measure` que remove uma unidade, mas SHALL bloquear a exclusão quando houver ao menos um produto usando a unidade (`Products.unit = name`), retornando erro com a contagem de produtos vinculados.

#### Scenario: Exclusão de unidade não utilizada
- **WHEN** o usuário exclui uma unidade que não é referenciada por nenhum produto
- **THEN** o sistema remove o registro de `Units_Of_Measure` e retorna sucesso

#### Scenario: Exclusão bloqueada por uso
- **WHEN** o usuário tenta excluir uma unidade referenciada por N produtos (N > 0)
- **THEN** o sistema retorna erro informando que a unidade está em uso por N produto(s) e não remove o registro

### Requirement: Seção de Unidades de Medida em Configurações

O sistema SHALL exibir uma seção "Unidades de Medida" no módulo de Configurações, com listagem das unidades (nome, sigla, ativo, inteiro), criação, edição de nome, sigla e inteiro, alternância de ativo e exclusão, espelhando o padrão da seção de Formas de Pagamento.

#### Scenario: Acesso à seção
- **WHEN** o usuário abre Configurações e seleciona "Unidades de Medida"
- **THEN** o sistema exibe a lista de unidades carregada por `list_units_of_measure`, indicando para cada unidade se é inteira

#### Scenario: Criar pela seção definindo inteiro
- **WHEN** o usuário preenche nome e sigla, escolhe se a unidade é inteira e confirma a criação
- **THEN** o sistema chama `create_unit_of_measure` com o `integer` escolhido e a nova unidade aparece na lista com esse estado

#### Scenario: Editar inteiro pela seção
- **WHEN** o usuário abre a edição de uma unidade, alterna o controle de inteiro e salva
- **THEN** o sistema chama `rename_unit_of_measure` com o novo `integer` e a lista reflete a alteração

#### Scenario: Erro de exclusão exibido
- **WHEN** a exclusão é bloqueada por uso
- **THEN** a seção exibe a mensagem de erro retornada pelo backend sem remover a unidade da lista
