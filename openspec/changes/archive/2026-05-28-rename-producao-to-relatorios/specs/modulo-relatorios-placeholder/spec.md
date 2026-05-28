## ADDED Requirements

### Requirement: Slot de Relatórios no dashboard
O dashboard SHALL exibir um módulo chamado "Relatórios" com identificador `"relatorios"`, substituindo o antigo módulo "Produção" (`"producao"`). O módulo SHALL usar um ícone de gráfico/relatório. O conteúdo da tela SHALL permanecer como placeholder até implementação futura.

#### Scenario: Módulo aparece no launcher
- **WHEN** o usuário abre o dashboard
- **THEN** o grid de módulos exibe "Relatórios" no lugar de "Produção"

#### Scenario: Clique abre o placeholder
- **WHEN** o usuário clica no card "Relatórios"
- **THEN** uma aba abre exibindo o nome do módulo como conteúdo (comportamento padrão de placeholder)
