## 1. Componente SplashScreen

- [ ] 1.1 Criar `src/components/SplashScreen.tsx` com logo (`cashier-svgrepo-com.svg`), nome "Teller" e prop `onFinished: () => void`
- [ ] 1.2 Adicionar animação de fade-in na montagem usando classes do `tw-animate-css`
- [ ] 1.3 Implementar estado `exiting` que ativa animação de fade-out e chama `onFinished` via `onAnimationEnd`

## 2. Integração em App.tsx

- [ ] 2.1 Substituir o bloco `screen === "loading"` pelo componente `<SplashScreen>`
- [ ] 2.2 Mover a lógica de `check_setup` para dentro da prop `onFinished`: determinar a tela destino e só então chamar `setScreen`

## 3. Verificação

- [ ] 3.1 Confirmar que a splash aparece com fade-in ao iniciar o app
- [ ] 3.2 Confirmar que a splash faz fade-out antes de exibir Login ou Setup
- [ ] 3.3 Executar `bun run build` sem erros TypeScript
