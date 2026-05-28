## Purpose

Tela principal pós-autenticação. Exibe a identidade do usuário logado e oferece ação de logout para retornar à tela de Login.

## Requirements

### Requirement: Exibição da identidade do usuário logado
O sistema SHALL exibir o nome e o apelido do usuário autenticado na tela de Dashboard após o login bem-sucedido.

#### Scenario: Usuário logado com nome e apelido
- **WHEN** o usuário acessa o Dashboard após autenticação
- **THEN** o header exibe o nome completo e o apelido do usuário logado

### Requirement: Ação de logout
O sistema SHALL permitir que o usuário encerre a sessão atual e retorne à tela de Login a partir do Dashboard.

#### Scenario: Logout executado
- **WHEN** o usuário clica no botão "Sair"
- **THEN** o estado do usuário autenticado é limpo e o app navega para a tela de Login

#### Scenario: Dashboard após logout
- **WHEN** o usuário retorna à tela de Login após logout
- **THEN** os campos de identificador e senha estão vazios

### Requirement: Lançador de módulos
O sistema SHALL exibir uma tela lançador como estado inicial do Dashboard, com saudação personalizada e grade de cards clicáveis para cada módulo disponível.

#### Scenario: Exibição do lançador ao entrar no Dashboard
- **WHEN** o usuário acessa o Dashboard após autenticação bem-sucedida
- **THEN** o lançador é exibido com a saudação "Olá, [nome]" e os cards dos módulos: Estoque, Compras, Relatórios, Vendas, Produtos, Configurações e Financeiro

#### Scenario: Retorno ao lançador pelo ícone home
- **WHEN** o usuário clica no ícone [🏠] na barra de abas
- **THEN** o lançador é exibido independentemente de quantas abas estejam abertas

### Requirement: Navegação por abas
O sistema SHALL permitir abrir módulos como abas na barra de navegação, com comportamento de foco e fechamento individual.

#### Scenario: Abertura de módulo via left-click no card
- **WHEN** o usuário faz left-click num card do lançador
- **THEN** se não existe aba desse módulo, uma nova aba é criada e ativada; se já existe, a aba existente é ativada

#### Scenario: Fechamento de aba pelo botão ×
- **WHEN** o usuário clica no × de uma aba
- **THEN** a aba é removida; se era a aba ativa, o sistema ativa a aba anterior ou exibe o lançador se não houver outras abas

#### Scenario: Troca de aba ativa
- **WHEN** o usuário clica numa aba inativa
- **THEN** essa aba torna-se a ativa e seu conteúdo é exibido

#### Scenario: Ctrl+W fecha a aba ativa
- **WHEN** o usuário pressiona Ctrl+W com uma aba ativa
- **THEN** a aba ativa é fechada com o mesmo comportamento do botão ×; sem efeito quando o lançador está ativo

#### Scenario: Ctrl+Tab cicla entre abas
- **WHEN** o usuário pressiona Ctrl+Tab com ao menos uma aba aberta
- **THEN** a próxima aba na sequência torna-se ativa, voltando à primeira ao atingir o fim

#### Scenario: Middle-click fecha aba
- **WHEN** o usuário clica com o botão do meio numa aba
- **THEN** essa aba é fechada com o mesmo comportamento do botão ×

### Requirement: Menu de contexto nos cards do lançador
O sistema SHALL exibir um menu de contexto ao clicar com o botão direito em um card do lançador.

#### Scenario: Right-click no card
- **WHEN** o usuário faz right-click num card do lançador
- **THEN** um menu de contexto é exibido com as opções "Abrir" e "Abrir nova instância"

#### Scenario: Opção "Abrir" no menu do card
- **WHEN** o usuário seleciona "Abrir" no menu de contexto do card
- **THEN** comportamento idêntico ao left-click: foca aba existente ou cria nova

#### Scenario: Opção "Abrir nova instância" no menu do card
- **WHEN** o usuário seleciona "Abrir nova instância" no menu de contexto do card
- **THEN** uma nova aba independente do módulo é criada e ativada, mesmo que já exista uma aba desse módulo

### Requirement: Menu de contexto nas abas
O sistema SHALL exibir um menu de contexto ao clicar com o botão direito numa aba aberta.

#### Scenario: Right-click na aba
- **WHEN** o usuário faz right-click numa aba na barra de abas
- **THEN** um menu de contexto é exibido com as opções "Duplicar" e "Fechar"

#### Scenario: Opção "Duplicar" no menu da aba
- **WHEN** o usuário seleciona "Duplicar" no menu de contexto da aba
- **THEN** uma nova aba do mesmo módulo é criada e ativada

#### Scenario: Opção "Fechar" no menu da aba
- **WHEN** o usuário seleciona "Fechar" no menu de contexto da aba
- **THEN** comportamento idêntico ao clique no × da aba
