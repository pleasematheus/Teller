## ADDED Requirements

### Requirement: Classificação de subtipo de despesa
O sistema SHALL permitir classificar cada lançamento financeiro com `expense_subtype` quando o tipo for `expense`. O campo SHALL ser opcional (null permitido para lançamentos sem classificação). Os valores válidos são: `despesa` (overhead fixo: aluguel, energia, papel), `custo` (operacional variável: pessoal, manutenção), `juros` (despesas financeiras), `imposto` (IR, CSLL), `devolucao` (devoluções de receita).

#### Scenario: Campo subtipo visível apenas para despesas
- **WHEN** o formulário de lançamento está no modo "Despesas" (tab Despesas ativa)
- **THEN** um select "Tipo de despesa" é exibido com as opções: Despesa, Custo, Juros, Imposto, Devolução e uma opção vazia (sem classificação)

#### Scenario: Campo subtipo oculto para receitas
- **WHEN** o formulário de lançamento está no modo "Receitas" (tab Receitas ativa)
- **THEN** o campo "Tipo de despesa" não é exibido

#### Scenario: Lançamento salvo com subtipo
- **WHEN** o usuário seleciona um subtipo e salva a despesa
- **THEN** o lançamento é persistido com `expense_subtype` preenchido e visível na lista como badge secundário

#### Scenario: Lançamento salvo sem subtipo
- **WHEN** o usuário deixa o campo subtipo em branco e salva a despesa
- **THEN** o lançamento é persistido com `expense_subtype = NULL` sem erro

## MODIFIED Requirements

### Requirement: Criação de lançamento financeiro
O sistema SHALL permitir criar um novo lançamento financeiro via formulário em sub-tab lateral. O tipo (receita/despesa) SHALL ser inferido da tab ativa no momento da abertura do formulário. Para lançamentos do tipo `expense`, o formulário SHALL incluir o campo opcional `expense_subtype` com as opções: `despesa`, `custo`, `juros`, `imposto`, `devolucao`.

#### Scenario: Abertura do formulário vazio
- **WHEN** o usuário clica em "Novo lançamento"
- **THEN** uma sub-tab de formulário abre com campos vazios e tipo pré-definido conforme a tab ativa

#### Scenario: Criação bem-sucedida
- **WHEN** o usuário preenche nome, categoria, valor e data de referência e confirma
- **THEN** o lançamento é salvo e a lista é atualizada sem fechar o módulo

#### Scenario: Criação com subtipo de despesa
- **WHEN** o usuário preenche o formulário de despesa, seleciona um `expense_subtype` e confirma
- **THEN** o lançamento é salvo com `expense_subtype` preenchido

#### Scenario: Validação de campos obrigatórios
- **WHEN** o usuário tenta confirmar sem preencher nome, valor ou data de referência
- **THEN** o formulário exibe erro nos campos ausentes e não salva

### Requirement: Edição de lançamento financeiro
O sistema SHALL permitir editar um lançamento existente com todos os campos pré-preenchidos no mesmo formulário de sub-tab, incluindo `expense_subtype` quando aplicável.

#### Scenario: Abertura do formulário de edição
- **WHEN** o usuário clica em editar um lançamento da lista
- **THEN** a sub-tab de formulário abre com todos os campos preenchidos com os valores atuais do lançamento

#### Scenario: Edição de subtipo
- **WHEN** o usuário altera o `expense_subtype` de um lançamento existente e confirma
- **THEN** o lançamento é atualizado com o novo subtipo

#### Scenario: Edição bem-sucedida
- **WHEN** o usuário altera algum campo e confirma
- **THEN** o lançamento é atualizado e a lista reflete os novos valores
