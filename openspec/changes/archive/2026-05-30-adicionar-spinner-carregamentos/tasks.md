## 1. Seções e listas

- [x] 1.1 `StockBalance.tsx`: adicionar `Spinner` à direita do texto "Carregando..." (linha ~81)
- [x] 1.2 `MovementHistory.tsx`: adicionar `Spinner` à direita do "Carregando..." (linha ~70)
- [x] 1.3 `PurchaseHistory.tsx`: adicionar `Spinner` à direita do "Carregando..." (linha ~33)
- [x] 1.4 `configuracoes/EmpresaSection.tsx`: adicionar `Spinner` à direita do "Carregando..." (linha ~72)
- [x] 1.5 `configuracoes/UsuariosSection.tsx`: adicionar `Spinner` à direita do "Carregando..." (linha ~133)
- [x] 1.6 `configuracoes/PagamentosSection.tsx`: adicionar `Spinner` aos dois estados "Carregando..." (métodos ~226 e taxas ~345)
- [x] 1.7 `relatorios/DreView.tsx`: exibir `Spinner` junto ao indicador de carregamento da tabela DRE

## 2. Botões de ação assíncrona

- [x] 2.1 `Login.tsx`: `Spinner` à direita de "Entrando..." mantendo `disabled` (linha ~129)
- [x] 2.2 `Setup.tsx`: `Spinner` à direita de "Criando..." mantendo `disabled` (linha ~122)
- [x] 2.3 `CompanySetup.tsx`: `Spinner` à direita de "Salvando..." mantendo `disabled` (linha ~130)

## 3. Selects

- [x] 3.1 `sales/SaleCloseForm.tsx`: renderizar `Spinner` adjacente ao select enquanto `methods.length === 0`, mantendo a option "Carregando..." (linha ~133)

## 4. Acessibilidade e verificação

- [x] 4.1 Passar `aria-hidden` ao `Spinner` nos pontos em que ele acompanha texto redundante de carregamento
- [x] 4.2 Conferir alinhamento (texto + spinner em container flex, gap consistente, `size-4`)
- [x] 4.3 Rodar `rtk bun run test` e `rtk tsc` para garantir que nada quebrou
