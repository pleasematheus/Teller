## Context

O `App.tsx` já tem o estado `"dashboard"` na state machine, mas renderiza apenas um parágrafo de placeholder. O `AuthUser` (id, name, nickname) está disponível no estado do `App`. A paleta e os tokens Tailwind do projeto já estão definidos.

## Goals / Non-Goals

**Goals:**
- Substituir o placeholder por uma tela funcional mínima
- Exibir identidade do usuário logado (name + nickname)
- Prover logout que retorna ao estado `"login"`
- Estabelecer a estrutura de layout (header fixo + área de conteúdo) que as próximas telas do ERP vão herdar

**Non-Goals:**
- Qualquer widget de dados real (vendas, estoque, relatórios)
- Navegação lateral ou menu entre seções
- Personalização ou configurações de usuário

## Decisions

### 1. Layout: header fixo + main
O Dashboard terá um header com logo, nome do usuário e botão de logout, e uma área `<main>` vazia para o conteúdo futuro. Essa estrutura é reutilizável — as próximas telas do ERP inserem conteúdo no `<main>` sem mudar o header.

**Alternativa descartada**: tela centrada como Login/Setup — não escala para um ERP com múltiplas seções.

### 2. Logout sem confirmação
Clicar em "Sair" limpa o estado do usuário e navega para `"login"` imediatamente, sem modal de confirmação.

**Alternativa descartada**: confirmação via dialog — desnecessária para uma sessão de turno onde logout é intencional.

### 3. Sem Tauri command para logout
Logout é puramente frontend — limpa estado React. Não há sessão persistida no backend para invalidar.
