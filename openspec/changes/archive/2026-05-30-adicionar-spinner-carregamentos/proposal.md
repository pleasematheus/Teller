## Why

Hoje os estados de carregamento são comunicados apenas por texto estático ("Carregando...", "Salvando...", "Entrando...", "Criando..."). Sem um indicador animado, o usuário não percebe que há atividade em andamento e a interface parece travada. Já existe um componente `Spinner` (`src/components/ui/spinner.tsx`), mas ele não é usado em nenhum lugar.

## What Changes

- Adicionar um indicador animado (`Spinner`) ao lado de todo texto de carregamento existente, posicionado à direita do texto.
- Padronizar os estados de carregamento de listas/seções: `StockBalance`, `MovementHistory`, `PurchaseHistory`, `EmpresaSection`, `UsuariosSection`, `PagamentosSection` (métodos e taxas), `DreView`.
- Padronizar os estados de carregamento de botões de ação: `Login` ("Entrando..."), `Setup` ("Criando..."), `CompanySetup` ("Salvando...").
- Padronizar o placeholder de carregamento de selects: `SaleCloseForm` ("Carregando...").
- Garantir que o `Spinner` exponha estado acessível (`role="status"`) e seja ocultado de leitores de tela quando acompanhado de texto redundante.

## Capabilities

### New Capabilities
- `loading-indicators`: Padrão visual de indicação de carregamento na interface, definindo onde e como um spinner animado acompanha textos de estado de carregamento (listas, seções, botões e selects).

### Modified Capabilities

## Impact

- Componente compartilhado: `src/components/ui/spinner.tsx` (já existe, passa a ser efetivamente utilizado).
- Componentes afetados: `StockBalance`, `MovementHistory`, `PurchaseHistory`, `EmpresaSection`, `UsuariosSection`, `PagamentosSection`, `DreView`, `Login`, `Setup`, `CompanySetup`, `SaleCloseForm`.
- Sem mudanças de backend, dependências ou contratos de dados. Mudança puramente de apresentação.
