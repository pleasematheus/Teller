# Spec: atualizacao-banco

## Purpose

Define o contrato do runner de migração embutido no binário principal: detecção de versão, backup antes de migrar, execução sequencial forward-only em transação, carimbo de versão, guarda de downgrade, suporte a salto de versões arbitrário e restauração em falha.

## Requirements

### Requirement: Detecção de versão na inicialização

O sistema SHALL, na inicialização, após abrir o banco e criar o schema em banco novo, ler `App_Meta.db_version` (versão armazenada) e compará-la com a constante `DB_VERSION` do binário (versão alvo) para decidir o fluxo de migração. O sistema SHALL NOT inferir a versão a partir das colunas presentes.

#### Scenario: Banco já na versão alvo
- **WHEN** o app inicia e `App_Meta.db_version` é igual a `DB_VERSION`
- **THEN** o sistema não executa nenhum passo de migração
- **THEN** a inicialização prossegue normalmente

#### Scenario: Banco novo carimbado não dispara migração
- **WHEN** o app cria um banco novo via `schema.sql` e carimba `db_version = DB_VERSION`
- **THEN** o runner de migração vê versão armazenada igual à alvo
- **THEN** nenhum passo de migração é executado

#### Scenario: Banco sem chave de versão é anômalo
- **WHEN** o app inicia sobre um banco existente que não contém a chave `db_version` em `App_Meta`
- **THEN** o sistema trata como erro de inicialização
- **THEN** o sistema não tenta inferir a versão nem migrar

### Requirement: Guarda de downgrade

O sistema SHALL recusar a inicialização quando a versão armazenada do banco for maior que `DB_VERSION` do binário, sem modificar o banco, expondo um erro claro de que o binário é mais antigo que o banco.

#### Scenario: Banco mais novo que o binário
- **WHEN** o app inicia e `App_Meta.db_version` é maior que `DB_VERSION`
- **THEN** o sistema recusa o boot
- **THEN** o sistema não executa nenhuma alteração no banco
- **THEN** o erro indica que o binário é mais antigo que o banco

### Requirement: Backup antes de migrar

O sistema SHALL, quando a versão armazenada for menor que a alvo, copiar o arquivo do banco para um backup nomeado com a versão de origem e um timestamp (`teller.db.bak.<stored>.<timestamp>`) uma única vez, antes de executar qualquer passo de migração. Se o backup não puder ser criado, o sistema SHALL abortar antes de tocar no banco.

#### Scenario: Backup criado antes do primeiro passo
- **WHEN** a versão armazenada é menor que a alvo
- **THEN** o sistema cria um arquivo de backup do banco antes de qualquer passo
- **THEN** o nome do backup inclui a versão de origem e um timestamp

#### Scenario: Falha ao criar backup aborta a migração
- **WHEN** o sistema não consegue criar o arquivo de backup
- **THEN** o sistema aborta a inicialização
- **THEN** o banco permanece inalterado

### Requirement: Execução sequencial forward-only com carimbo de versão

O sistema SHALL executar os passos de migração em ordem crescente, de `stored+1` até `target` inclusive, cada passo numa transação que ao concluir atualiza `App_Meta.db_version` para a versão daquele passo. v6 é a versão baseline; o sistema SHALL NOT prover passos para versões anteriores a 6.

#### Scenario: Aplicação de um único passo
- **WHEN** a versão armazenada é `N-1` e a alvo é `N`
- **THEN** o sistema executa o passo `N`
- **THEN** ao concluir, `App_Meta.db_version` é igual a `N`

#### Scenario: Ordem crescente preservada
- **WHEN** múltiplos passos precisam ser aplicados
- **THEN** o sistema os executa em ordem crescente de versão
- **THEN** cada passo só executa após o anterior ter concluído e carimbado sua versão

### Requirement: Salto de versões arbitrário numa execução

O sistema SHALL migrar um banco que esteja várias versões atrás até a versão alvo numa única inicialização, aplicando todos os passos intermediários em ordem, sem exigir versões intermediárias do binário. O binário SHALL conter todos os passos de migração até seu `DB_VERSION`.

#### Scenario: Banco dormente salta várias versões
- **WHEN** a versão armazenada é 6 e a alvo é 20
- **THEN** o sistema aplica os passos de 7 a 20 em ordem numa única execução
- **THEN** ao concluir, `App_Meta.db_version` é igual a 20

### Requirement: Restauração e aborto em falha de passo

O sistema SHALL, ao falhar qualquer passo de migração, tentar restaurar o backup, abortar a inicialização e expor o erro. O sistema SHALL recusar a restauração automática se houver um WAL pendente. O sistema SHALL preparar uma cópia completa do backup antes de substituir o banco. Se a restauração falhar, o sistema SHALL expor também esse erro e o caminho do backup. O sistema SHALL NOT abrir os módulos de gestão após uma falha.

#### Scenario: Falha no meio de uma cadeia de passos
- **WHEN** uma migração de 6 para 20 falha no passo 15
- **THEN** o sistema restaura o backup criado na versão 6
- **THEN** o sistema aborta a inicialização e expõe o erro
- **THEN** os dados originais permanecem intactos

#### Scenario: WAL pendente impede a restauração automática
- **WHEN** existe um WAL ao iniciar a restauração
- **THEN** o sistema mantém o banco, o WAL e o backup
- **THEN** o sistema recusa a restauração e informa o erro

#### Scenario: Falha ao ler ou substituir o backup
- **WHEN** a leitura do backup ou a substituição do banco falha
- **THEN** o sistema aborta a inicialização
- **THEN** o sistema informa o erro da migração e o erro da restauração
- **THEN** o sistema mantém o backup para recuperação

### Requirement: Instalação nova não executa migrações

O sistema SHALL criar bancos novos diretamente a partir de `schema.sql` (sempre a versão corrente) e SHALL NOT executar nenhum passo de migração para bancos recém-criados.

#### Scenario: Instalação limpa pula migrações
- **WHEN** o app é instalado e inicia sobre um banco vazio
- **THEN** o sistema cria o schema via `schema.sql` e carimba `db_version = DB_VERSION`
- **THEN** nenhum passo de migração é executado

### Requirement: Migração v8 adiciona is_cash a Payment_Methods

O sistema SHALL incluir, na lista `MIGRATIONS`, um passo direcionado à versão 8 que adiciona a coluna `is_cash BOOLEAN NOT NULL DEFAULT false` à tabela `Payment_Methods`, e SHALL incrementar `DB_VERSION` para `"8"`. O passo SHALL ser idempotente em relação a instalações novas (cujo `schema.sql` já cria a coluna).

#### Scenario: Banco existente em v7 migra para v8
- **WHEN** o app inicia sobre um banco carimbado na versão 7
- **THEN** o passo de migração v8 adiciona a coluna `is_cash` a `Payment_Methods`, os métodos existentes ficam com `is_cash = false` e a versão é carimbada como 8

#### Scenario: Instalação nova já em v8
- **WHEN** um banco é criado do zero a partir do `schema.sql` atual
- **THEN** `Payment_Methods` já contém a coluna `is_cash` e a versão é carimbada como 8 sem executar o passo de migração
