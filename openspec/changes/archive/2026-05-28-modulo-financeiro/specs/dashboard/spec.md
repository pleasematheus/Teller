## MODIFIED Requirements

### Requirement: Lançador de módulos
O sistema SHALL exibir uma tela lançador como estado inicial do Dashboard, com saudação personalizada e grade de cards clicáveis para cada módulo disponível.

#### Scenario: Exibição do lançador ao entrar no Dashboard
- **WHEN** o usuário acessa o Dashboard após autenticação bem-sucedida
- **THEN** o lançador é exibido com a saudação "Olá, [nome]" e os cards dos módulos: Estoque, Compras, Relatórios, Vendas, Produtos, Configurações e Financeiro

#### Scenario: Retorno ao lançador pelo ícone home
- **WHEN** o usuário clica no ícone [🏠] na barra de abas
- **THEN** o lançador é exibido independentemente de quantas abas estejam abertas
