## Purpose

Módulo de gestão financeira do Teller. Permite registrar, editar e excluir lançamentos financeiros (receitas e despesas) classificados por mês de competência, com distinção entre custos fixos e variáveis, sugestões automáticas para entradas recorrentes e filtro mensal com navegação por seletor de mês.

## Requirements

### Requirement: Visualização de lançamentos por mês e tipo
O sistema SHALL exibir os lançamentos financeiros separados em duas tabs ("Receitas" e "Despesas") com filtro por mês via seletor de calendário. Apenas entradas cujo `reference_date` pertence ao mês selecionado SHALL ser exibidas.

#### Scenario: Abertura do módulo exibe mês atual
- **WHEN** o usuário abre o módulo Financeiro
- **THEN** a tab "Despesas" está ativa e o mês selecionado é o mês corrente

#### Scenario: Troca de tab filtra por tipo
- **WHEN** o usuário clica na tab "Receitas"
- **THEN** a lista exibe apenas entradas com `type = 'income'` do mês selecionado

#### Scenario: Troca de mês atualiza lista
- **WHEN** o usuário seleciona um mês diferente no seletor
- **THEN** a lista é recarregada com as entradas do novo mês para a tab ativa

### Requirement: Criação de lançamento financeiro
O sistema SHALL permitir criar um novo lançamento financeiro via formulário em sub-tab lateral. O tipo (receita/despesa) SHALL ser inferido da tab ativa no momento da abertura do formulário.

#### Scenario: Abertura do formulário vazio
- **WHEN** o usuário clica em "Novo lançamento"
- **THEN** uma sub-tab de formulário abre com campos vazios e tipo pré-definido conforme a tab ativa

#### Scenario: Criação bem-sucedida
- **WHEN** o usuário preenche nome, categoria, valor e data de referência e confirma
- **THEN** o lançamento é salvo e a lista é atualizada sem fechar o módulo

#### Scenario: Validação de campos obrigatórios
- **WHEN** o usuário tenta confirmar sem preencher nome, valor ou data de referência
- **THEN** o formulário exibe erro nos campos ausentes e não salva

### Requirement: Edição de lançamento financeiro
O sistema SHALL permitir editar um lançamento existente com todos os campos pré-preenchidos no mesmo formulário de sub-tab.

#### Scenario: Abertura do formulário de edição
- **WHEN** o usuário clica em editar um lançamento da lista
- **THEN** a sub-tab de formulário abre com todos os campos preenchidos com os valores atuais do lançamento

#### Scenario: Edição bem-sucedida
- **WHEN** o usuário altera algum campo e confirma
- **THEN** o lançamento é atualizado e a lista reflete os novos valores

### Requirement: Exclusão de lançamento financeiro
O sistema SHALL permitir excluir um lançamento individual da lista.

#### Scenario: Exclusão confirmada
- **WHEN** o usuário confirma a exclusão de um lançamento
- **THEN** o registro é removido do banco e desaparece da lista

### Requirement: Lançamentos recorrentes — sugestões mensais
O sistema SHALL detectar lançamentos marcados como `is_recurring = true` em meses anteriores que ainda não foram registrados no mês atual e exibi-los como sugestões na seção "Sugestões recorrentes". A sugestão SHALL pré-preencher o formulário com nome, categoria, valor e `is_fixed` do registro anterior, todos editáveis antes de salvar.

#### Scenario: Sugestões exibidas para mês atual sem registro correspondente
- **WHEN** existe ao menos um lançamento recorrente de mês anterior sem correspondente no mês selecionado
- **THEN** a seção "Sugestões recorrentes" aparece acima da lista com um card por sugestão

#### Scenario: Nenhuma sugestão pendente
- **WHEN** todos os lançamentos recorrentes já foram registrados no mês selecionado
- **THEN** a seção "Sugestões recorrentes" não é exibida

#### Scenario: Registro a partir de sugestão
- **WHEN** o usuário clica em "Registrar" em uma sugestão
- **THEN** o formulário abre pré-preenchido com os dados da sugestão; ao confirmar, o lançamento é salvo e a sugestão some da seção

### Requirement: Distinção entre custo fixo e variável
O sistema SHALL permitir marcar cada lançamento como fixo (`is_fixed = true`) ou variável (`is_fixed = false`) via toggle explícito no formulário. Esta classificação SHALL ser usada futuramente pelo módulo de Relatórios para o cálculo de DF e CV no DRE.

#### Scenario: Toggle "Custo fixo" no formulário
- **WHEN** o usuário abre o formulário de criação ou edição
- **THEN** existe um toggle "Custo fixo" desativado por padrão

#### Scenario: Lançamento salvo com flag fixo
- **WHEN** o usuário ativa "Custo fixo" e salva
- **THEN** o lançamento é persistido com `is_fixed = true` e exibido com badge "Fixo" na lista
