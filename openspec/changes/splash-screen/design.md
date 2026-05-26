## Context

O App.tsx atualmente renderiza um `<div>` inline com texto "Carregando..." enquanto `check_setup` não retorna. Esse estado dura tipicamente 50–200ms — curto demais para uma tela de conteúdo, longo o suficiente para uma transição brusca. O componente `SplashScreen` substitui esse bloco com identidade visual consistente com o restante do app.

## Goals / Non-Goals

**Goals:**
- Exibir logo e nome do app durante o carregamento inicial
- Animação de fade-in na entrada e fade-out na saída
- Visual consistente com o design system existente (tokens `teller-*`)

**Non-Goals:**
- Barra de progresso ou indicador de etapas de carregamento
- Splash screen nativa via `tauri.conf.json` (janela separada)
- Preload de dados além do que já ocorre em `check_setup`
- Delay artificial para garantir que a splash seja vista

## Decisions

### Animação: CSS + Tailwind, sem biblioteca externa

`tw-animate-css` já está instalado no projeto e fornece `animate-fade-in` e classes equivalentes. A saída (fade-out antes de navegar) será controlada por estado local: quando `check_setup` retorna, ativa a animação de saída e aguarda sua duração antes de mudar a tela.

Alternativa descartada: `framer-motion` — dependência pesada desnecessária para uma animação simples.

### Controle do fade-out: estado local + `onAnimationEnd`

```
check_setup resolve → setExiting(true) → CSS fade-out inicia
→ onAnimationEnd → App chama setScreen(destino)
```

O `SplashScreen` recebe uma prop `onFinished` que é chamada ao término da animação de saída. O App.tsx passa a tela destino para essa callback após `check_setup` resolver.

### Estrutura: componente único sem subcomponentes

`SplashScreen.tsx` é auto-contido — reutiliza assets já presentes (`cashier-svgrepo-com.svg`).

## Risks / Trade-offs

- [Animação de saída vs tempo real] Se `check_setup` demorar mais que o esperado, o usuário vê a splash por tempo indeterminado → Mitigation: sem delay artificial; a splash dura exatamente o tempo do `check_setup` + duração do fade-out (~300ms)
- [onAnimationEnd não dispara] Em ambientes sem animação (redução de movimento, testes) → Mitigation: usar `duration-300` curto; aceitável para MVP
