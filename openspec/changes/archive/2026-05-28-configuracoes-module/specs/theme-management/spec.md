## ADDED Requirements

### Requirement: Seleção de tema no módulo de Configurações
O sistema SHALL oferecer um seletor de tema compacto no painel esquerdo do módulo de Configurações, usando o componente `Tabs` com variante `default` (pills), aplicando e persistindo a escolha com o mesmo comportamento do ThemeToggle existente no TabBar.

#### Scenario: Seletor exibe tema atual
- **WHEN** o usuário abre o módulo de Configurações
- **THEN** o tab correspondente ao tema ativo do usuário está selecionado (Claro, Escuro ou Sistema)

#### Scenario: Selecionar Escuro via Configurações
- **WHEN** o usuário seleciona "Escuro" no seletor de tema das Configurações
- **THEN** o tema dark é aplicado imediatamente e salvo no banco — mesmo comportamento do ThemeToggle no TabBar

#### Scenario: Selecionar Claro via Configurações
- **WHEN** o usuário seleciona "Claro" no seletor de tema das Configurações
- **THEN** o tema light é aplicado imediatamente e salvo no banco

#### Scenario: Selecionar Sistema via Configurações
- **WHEN** o usuário seleciona "Sistema" no seletor de tema das Configurações
- **THEN** o tema do sistema operacional é aplicado imediatamente e salvo no banco

#### Scenario: ThemeToggle no TabBar permanece funcional
- **WHEN** o usuário altera o tema pelo ThemeToggle no TabBar
- **THEN** o seletor nas Configurações reflete o novo tema (ambos compartilham o mesmo estado)
