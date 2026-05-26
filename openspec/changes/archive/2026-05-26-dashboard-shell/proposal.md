## Why

O Dashboard atual é uma casca vazia — exibe usuário e logout, mas não oferece navegação para nenhuma funcionalidade. Este change implementa o shell completo: uma barra de abas estilo navegador e uma tela lançador com cards de módulos, permitindo que as futuras features de cada módulo sejam integradas sem redesenhar a navegação.

## What Changes

- Adiciona barra de abas linha única no header (substituindo o header atual)
- Adiciona tela lançador (home) com grid 3×2 de cards de módulos
- Saudação com nome do usuário na tela lançador
- Comportamento de clique nos cards: esquerdo abre/foca aba, direito exibe menu de contexto
- Menu de contexto no card: "Abrir" e "Abrir nova instância"
- Menu de contexto na aba: "Duplicar" e "Fechar"
- Ícone `[🏠]` como âncora permanente (sem ×) para retornar ao lançador

## Capabilities

### New Capabilities

_(nenhuma — mudanças são aditivas sobre a capability `dashboard` existente)_

### Modified Capabilities

- `dashboard`: adiciona requisitos de lançador, tab navigation e menus de contexto; os requisitos existentes (identidade do usuário, logout) permanecem válidos mas migram para a nova estrutura do header

## Impact

- `src/components/Dashboard.tsx`: refatoração completa do componente
- Novos subcomponentes: `TabBar`, `Launcher`, `ModuleCard`
- Estado interno do Dashboard: lista de abas abertas, aba ativa
- Nenhuma mudança no backend ou na lógica de autenticação
