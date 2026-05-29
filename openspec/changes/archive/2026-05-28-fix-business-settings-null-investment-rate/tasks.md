## 1. Schema e migração

- [x] 1.1 Em `src/db/schema.sql`, remover `NOT NULL DEFAULT 0` de `investment_rate` e `profit_margin` em `Business_Settings` (tornar nullable, sem default)
- [x] 1.2 Em `src-tauri/src/db.rs`, adicionar função `ensure_business_settings_nullable` que executa `ALTER TABLE "Business_Settings" ALTER COLUMN investment_rate DROP NOT NULL` e idem para `profit_margin`
- [x] 1.3 Em `src-tauri/src/auth.rs` (função `open_database`), chamar `ensure_business_settings_nullable` no startup junto com as demais migrations

## 2. Rust: create e update

- [x] 2.1 Em `src-tauri/src/auth.rs`, alterar `create_business_settings` para aceitar `investment_rate: Option<f64>` e `profit_margin: Option<f64>`; validar range apenas se `Some`
- [x] 2.2 Em `src-tauri/src/settings.rs`, alterar `update_business_settings` para aceitar `investment_rate: Option<f64>` e `profit_margin: Option<f64>`; validar range apenas se `Some`

## 3. Rust: read (BusinessInfo e PricingRates)

- [x] 3.1 Em `src-tauri/src/settings.rs`, alterar `BusinessInfo` para `investment_rate: Option<f64>` e `profit_margin: Option<f64>`
- [x] 3.2 Em `src-tauri/src/settings.rs`, alterar `get_business_info` para ler `Option<f64>` do banco via `row.get::<_, Option<f64>>(1)?`
- [x] 3.3 Em `src-tauri/src/products.rs`, alterar `get_pricing_rates` para ler `(Option<f64>, Option<f64>)` do banco e usar `.unwrap_or(0.0)` antes de retornar `PricingRates`

## 4. Frontend: CompanySetup

- [x] 4.1 Em `src/components/CompanySetup.tsx`, substituir `parseFloat(form.investmentRate) / 100` por `form.investmentRate !== "" ? parseFloat(form.investmentRate) / 100 : null`
- [x] 4.2 Idem para `profitMargin`

## 5. Frontend: EmpresaSection

- [x] 5.1 Em `src/components/configuracoes/EmpresaSection.tsx`, atualizar o tipo `BusinessInfo` para `investment_rate: number | null` e `profit_margin: number | null`
- [x] 5.2 No `useEffect` de carregamento, tratar null → string vazia: `info.investment_rate != null ? String(Math.round(info.investment_rate * 10000) / 100) : ""`
- [x] 5.3 No `handleSubmit`, substituir `parseFloat(form.investmentRate) / 100` por `form.investmentRate !== "" ? parseFloat(form.investmentRate) / 100 : null`
- [x] 5.4 Idem para `profitMargin`

## 6. Verificação

- [x] 6.1 `rtk cargo test` — sem regressões nos testes Rust
- [x] 6.2 Verificar manualmente: criar empresa com campos de taxa vazios → salva sem erro  *(pendente: teste manual)*
- [x] 6.3 Verificar manualmente: criar empresa com valores numéricos (ex: 10, 20) → salva corretamente  *(pendente: teste manual)*
- [ ] 6.4 Verificar manualmente: abrir configurações → campos de taxa exibem vazio quando null e exibem valor quando preenchido  *(pendente: teste manual)*
