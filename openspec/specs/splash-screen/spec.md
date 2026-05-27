## Purpose

Tela de splash exibida durante o carregamento inicial do app, substituindo indicadores genéricos por identidade visual consistente com o design system.

## Requirements

### Requirement: Exibição da splash screen durante carregamento inicial
O sistema SHALL exibir uma tela de splash com identidade visual do app enquanto a verificação de setup está em andamento, substituindo qualquer indicador de carregamento genérico.

#### Scenario: Splash exibida durante check_setup
- **WHEN** o app inicia e `check_setup` ainda não retornou
- **THEN** a splash screen é exibida com o logo e o nome "Teller" sobre o fundo do tema

#### Scenario: Splash com animação de entrada
- **WHEN** a splash screen é montada
- **THEN** o conteúdo aparece com animação de fade-in

### Requirement: Transição de saída da splash screen
O sistema SHALL remover a splash screen com animação de saída antes de exibir a tela destino, garantindo uma transição suave.

#### Scenario: Fade-out ao concluir carregamento
- **WHEN** `check_setup` retorna e a tela destino é determinada
- **THEN** a splash screen executa animação de fade-out antes de ceder lugar à tela seguinte

#### Scenario: Navegação ocorre após animação
- **WHEN** a animação de saída da splash screen termina
- **THEN** a tela destino (Setup ou Login) é exibida
