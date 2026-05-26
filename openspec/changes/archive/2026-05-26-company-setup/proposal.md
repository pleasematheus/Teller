## Why

O sistema já coleta dados do usuário no Setup inicial, mas não tem contexto da empresa operada — nome, taxa de investimento e margem de lucro necessários para cálculos de precificação. A tabela `Business_Settings` foi adicionada ao schema; agora é preciso o fluxo que a popula antes do Dashboard.

## What Changes

- **Nova tela de cadastro de empresa**: após o primeiro login bem-sucedido, se `Business_Settings` estiver vazia, o app exibe um formulário de cadastro com nome da empresa, taxa de investimento e margem de lucro.
- **Novo estado de navegação `"company-setup"`**: inserido entre login e dashboard na state machine do `App.tsx`.
- **Dois novos comandos Tauri**: `check_business_setup` (verifica se há registro em `Business_Settings`) e `create_business_settings` (insere o primeiro registro).
- **Correção do guard do `init_schema`**: o guard atual usa `Financial_Entries` como sentinela, que já existe em bancos antigos — esses bancos nunca receberiam as novas tabelas. O guard passa a verificar `Production_Inputs` (última tabela do novo schema), garantindo que bancos criados com o schema anterior sejam migrados.

## Capabilities

### New Capabilities

- `company-setup`: fluxo de cadastro de dados da empresa na primeira utilização — verificação de existência, formulário e persistência em `Business_Settings`

### Modified Capabilities

- `state-navigation`: adição do estado `"company-setup"` à state machine e das transições Login → Company Setup → Dashboard

## Impact

- `src-tauri/src/lib.rs`: função `init_schema` (guard atualizado), dois novos comandos Tauri, registro em `invoke_handler`
- `src/App.tsx`: type `Screen` expandido, `useEffect` pós-login, novo ramo de renderização
- `src/components/CompanySetup.tsx`: novo componente (criar)
