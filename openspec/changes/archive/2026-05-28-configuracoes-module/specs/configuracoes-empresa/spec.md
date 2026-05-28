## ADDED Requirements

### Requirement: Carregar dados atuais da empresa
O sistema SHALL carregar os dados atuais de `Business_Settings` ao abrir a seção Empresa, exibindo os valores vigentes nos campos do formulário.

#### Scenario: Dados carregados ao abrir seção
- **WHEN** o usuário abre a seção Empresa no módulo de Configurações
- **THEN** o formulário exibe o nome da empresa, taxa de investimento e margem de lucro atuais lidos do banco

#### Scenario: Campos preenchidos com valores existentes
- **WHEN** `Business_Settings` contém `name = "Padaria do João"`, `investment_rate = 0.10`, `profit_margin = 0.20`
- **THEN** o campo nome exibe "Padaria do João", taxa exibe "10", margem exibe "20"

### Requirement: Editar e salvar dados da empresa
O sistema SHALL permitir ao usuário alterar nome da empresa, taxa de investimento (%) e margem de lucro (%) e persistir as alterações via command `update_business_settings`.

#### Scenario: Salvar alterações com dados válidos
- **WHEN** o usuário edita os campos e clica em "Salvar" com nome não-vazio e taxas no intervalo [0, 100]
- **THEN** o sistema chama `update_business_settings` e exibe confirmação de sucesso

#### Scenario: Rejeitar nome vazio
- **WHEN** o usuário deixa o campo nome vazio e clica em "Salvar"
- **THEN** o sistema exibe mensagem de erro e não persiste

#### Scenario: Rejeitar taxa fora do intervalo
- **WHEN** o usuário informa taxa de investimento ou margem de lucro fora de [0, 100]
- **THEN** o sistema exibe mensagem de erro e não persiste

### Requirement: Command update_business_settings
O sistema SHALL expor um Tauri command `update_business_settings` que atualiza o registro único de `Business_Settings` com os novos valores.

#### Scenario: Atualização bem-sucedida
- **WHEN** `update_business_settings` é chamado com nome válido e taxas no intervalo [0.0, 1.0] (decimais)
- **THEN** o registro em `Business_Settings` é atualizado e `updated_at` é renovado

#### Scenario: Rejeitar nome vazio no backend
- **WHEN** `update_business_settings` é chamado com nome vazio ou apenas espaços
- **THEN** o command retorna `Err("Nome da empresa não pode ser vazio")`

#### Scenario: Rejeitar taxas fora do intervalo no backend
- **WHEN** `update_business_settings` é chamado com `investment_rate` ou `profit_margin` fora de [0.0, 1.0]
- **THEN** o command retorna `Err` com mensagem descritiva
