## 1. SaleCloseForm

- [x] 1.1 No seletor de parcelas, alterar o rótulo da opção de `{fee.installments}x ({(fee.fee_percent * 100).toFixed(2)}%)` para `{fee.installments}x` (manter `value={fee.installments}`)
- [x] 1.2 Remover o bloco JSX "você recebe: {brl.format(netAmount)}" e o cálculo de `netAmount`
- [x] 1.3 Remover a função `computeNetAmount` (não mais usada)

## 2. Verificação

- [x] 2.1 `bun run test` e `tsc` passam sem erros (sem variáveis/imports não usados)
- [x] 2.2 Conferir no app: recebimento mostra parcelas sem `%`, sem linha de valor líquido; total e restante intactos; fechar venda parcelada ainda registra `installments` e `net_amount` corretos
