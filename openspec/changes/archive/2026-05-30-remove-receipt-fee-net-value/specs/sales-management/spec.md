## MODIFIED Requirements

### Requirement: Seleção de modalidade e parcela no fechamento
O sistema SHALL permitir, no fechamento, escolher para cada pagamento o método e — quando o método tiver taxas por parcela — o número de parcelas. A UI do recebimento NÃO SHALL exibir a porcentagem da taxa nas opções de parcela nem o valor líquido recebido; a parcela escolhida continua determinando a taxa registrada no backend.

#### Scenario: Método com tabela de parcelas
- **WHEN** o operador seleciona um método que possui taxas configuradas
- **THEN** o sistema exibe as opções de parcela apenas pelo número de parcelas (ex.: "1x", "2x"), sem a porcentagem da taxa em cada opção

#### Scenario: Método sem parcelamento
- **WHEN** o operador seleciona um método sem taxa (ex.: Pix, dinheiro)
- **THEN** o sistema não exibe seletor de parcelas e usa `installments = 1`

#### Scenario: Valor líquido não exibido no recebimento
- **WHEN** o operador informa um valor para um pagamento com método que possui taxas
- **THEN** o sistema não exibe o valor líquido recebido ("você recebe") na tela de recebimento

#### Scenario: Parcela escolhida define a taxa registrada
- **WHEN** o operador confirma o pagamento com uma parcela escolhida
- **THEN** o `installments` escolhido é enviado ao `close_sale` e determina a taxa aplicada ao `net_amount`
