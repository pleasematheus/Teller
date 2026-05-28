## Purpose

Slot de navegação "Relatórios" no dashboard do Teller. Reserva o card e o `ModuleId` para o futuro módulo de relatórios (DRE, Sankey, etc.) sem implementar conteúdo ainda.

## Requirements

### Requirement: Slot de Relatórios no dashboard
O dashboard SHALL exibir um módulo chamado "Relatórios" com identificador `"relatorios"`, substituindo o antigo módulo "Produção" (`"producao"`). O módulo SHALL usar um ícone de gráfico/relatório. O conteúdo da tela SHALL permanecer como placeholder até implementação futura.

#### Scenario: Módulo aparece no launcher
- **WHEN** o usuário abre o dashboard
- **THEN** o grid de módulos exibe "Relatórios" no lugar de "Produção"

#### Scenario: Clique abre o placeholder
- **WHEN** o usuário clica no card "Relatórios"
- **THEN** uma aba abre exibindo o texto "Relatórios — em breve" como conteúdo
