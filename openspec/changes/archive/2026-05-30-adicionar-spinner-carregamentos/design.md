## Context

Os estados de carregamento da aplicação são comunicados hoje apenas por texto: seções renderizam `<p>Carregando...</p>`, botões trocam o rótulo para "Entrando..."/"Criando..."/"Salvando...", e `SaleCloseForm` mostra uma `<option>Carregando...</option>`. Já existe `src/components/ui/spinner.tsx`, baseado em `HugeiconsIcon` com `Loading03Icon`, `animate-spin`, `size-4` e `role="status"`, porém não é importado em lugar nenhum. A mudança é transversal a vários componentes, mas puramente de apresentação.

## Goals / Non-Goals

**Goals:**
- Reutilizar o `Spinner` existente em todos os pontos de carregamento, sempre à direita do texto.
- Estabelecer um padrão de marcação consistente (texto + spinner em um container flex) para reuso futuro.
- Preservar acessibilidade: evitar anúncio duplicado quando há texto descritivo.

**Non-Goals:**
- Não criar skeletons, barras de progresso ou overlays de tela cheia.
- Não alterar lógica de fetch, timing ou comportamento de negócio.
- Não substituir o `SplashScreen` (já tem sua própria animação).
- Não adicionar dependências novas.

## Decisions

- **Reusar o componente `Spinner` existente, sem reescrevê-lo.** O componente já entrega ícone animado e `role="status"`. Alternativa descartada: criar um novo componente de "loading row" — desnecessário dado que o `Spinner` já existe e o agrupamento texto+spinner é trivial com flex.
- **Posicionar o spinner à direita do texto** dentro de um container `inline-flex items-center gap-2` (ou equivalente já presente). Isso cumpre o pedido explícito do usuário ("um spinner a direita") e mantém alinhamento vertical com o texto.
- **Marcar o ícone como `aria-hidden` quando acompanhado de texto redundante.** O `Spinner` traz `aria-label="Loading"` e `role="status"`; ao lado de um texto "Carregando..." isso gera anúncio duplicado. Passar `aria-hidden` via props nesses casos mantém o texto como fonte única do estado. Em botões, o próprio texto ("Entrando...") já cumpre o papel de status.
- **Em botões**, inserir o `Spinner` ao lado do label de progresso mantendo o `disabled` já existente. Não mudar a lógica de `disabled`.
- **Em selects** (`SaleCloseForm`), como não é possível embutir um SVG dentro de `<option>`, renderizar o spinner ao lado do controle (à direita do select) enquanto as opções não chegaram, mantendo a `<option>Carregando...</option>` como fallback textual.

## Risks / Trade-offs

- [Spinner dentro de `<option>` não é suportado pelo HTML] → Renderizar o spinner adjacente ao select, não dentro da option.
- [Inconsistência de tamanho/cor entre os pontos] → Usar o tamanho padrão `size-4` do `Spinner` e herdar `currentColor`; só ajustar via `className` quando o contexto (botão escuro) exigir.
- [Anúncio redundante em leitores de tela] → Mitigado marcando o ícone como decorativo quando há texto.

## Migration Plan

Mudança incremental por componente; cada edição é isolada e reversível. Sem migração de dados nem flags. Rollback = reverter os imports/JSX adicionados.
