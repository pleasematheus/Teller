## 1. Backend Rust (lib.rs)

- [x] 1.1 Atualizar o guard do `init_schema`: trocar a verificação de `Financial_Entries` por `Production_Inputs`
- [x] 1.2 Adicionar comando `check_business_setup` que retorna `Result<bool, String>` (COUNT(*) > 0 em `Business_Settings`)
- [x] 1.3 Adicionar comando `create_business_settings(name, investment_rate, profit_margin)` com validações: nome não-vazio, taxas no intervalo [0.0, 1.0]
- [x] 1.4 Registrar `check_business_setup` e `create_business_settings` no `invoke_handler`

## 2. Testes Rust

- [x] 2.1 Adicionar teste `check_business_setup_empty_returns_false` (banco sem registro retorna false)
- [x] 2.2 Adicionar teste `check_business_setup_with_record_returns_true` (banco com registro retorna true)
- [x] 2.3 Adicionar teste `create_business_settings_rejects_empty_name`
- [x] 2.4 Adicionar teste `create_business_settings_rejects_rate_out_of_range`
- [x] 2.5 Executar `cargo test` e confirmar que todos os testes passam

## 3. Frontend — Componente CompanySetup

- [x] 3.1 Criar `src/components/CompanySetup.tsx` com formulário: campo "Nome da empresa" (text), "Taxa de investimento %" (number, 0–100), "Margem de lucro %" (number, 0–100)
- [x] 3.2 No submit: converter os percentuais para fração (÷ 100) antes de invocar `create_business_settings`
- [x] 3.3 Exibir erro inline se o comando retornar Err
- [x] 3.4 Chamar `onComplete()` ao receber sucesso

## 4. Frontend — App.tsx

- [x] 4.1 Adicionar `"company-setup"` ao type `Screen`
- [x] 4.2 Após `onSuccess` do Login: invocar `check_business_setup` e navegar para `"company-setup"` ou `"dashboard"` conforme o resultado
- [x] 4.3 Adicionar ramo de renderização para `screen === "company-setup"` que exibe `<CompanySetup onComplete={() => setScreen("dashboard")} />`
- [x] 4.4 Passar `user` para `CompanySetup` via prop (para exibir o nome no cabeçalho, se desejado)
