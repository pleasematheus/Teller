## Why

Ao submeter o formulário de criação de empresa com os campos `investmentRate` e `profitMargin` vazios, `parseFloat("")` retorna `NaN`, que Tauri serializa como `null`, e o Rust rejeita com erro interno. Além do crash, "vazio" e "zero" têm significados diferentes de negócio: zero significa taxa configurada em 0%; vazio significa "ainda não configurei". O fix correto é suportar `null` de ponta a ponta.

## What Changes

- `Business_Settings` no banco: remover `NOT NULL` de `investment_rate` e `profit_margin` (aceitar NULL).
- Migração para DBs existentes: `ALTER TABLE ... ALTER COLUMN ... DROP NOT NULL`.
- Rust `create_business_settings` e `update_business_settings`: aceitar `Option<f64>`; validar range apenas se `Some`.
- Rust `BusinessInfo`: `investment_rate: Option<f64>`, `profit_margin: Option<f64>`.
- Rust `get_pricing_rates`: tratar `NULL` no banco como `0.0` ao calcular markup (sem impacto no cálculo).
- Frontend `CompanySetup.tsx` e `EmpresaSection.tsx`: enviar `null` quando campo vazio; exibir campo vazio quando valor é `null`.

## Capabilities

### New Capabilities

_(nenhuma nova capability)_

### Modified Capabilities

- `company-setup`: campos numéricos de taxa passam a ser opcionais (nullable), com comportamento distinto de zero.

## Impact

- `src/db/schema.sql`: colunas `investment_rate` e `profit_margin` em `Business_Settings` passam a ser nullable.
- `src-tauri/src/db.rs`: nova função de migração para remover NOT NULL em DBs existentes.
- `src-tauri/src/auth.rs`: `create_business_settings` com `Option<f64>`.
- `src-tauri/src/settings.rs`: `BusinessInfo`, `get_business_info`, `update_business_settings` com `Option<f64>`.
- `src-tauri/src/products.rs`: `get_pricing_rates` lê NULL como 0.0.
- `src/components/CompanySetup.tsx` e `src/components/configuracoes/EmpresaSection.tsx`: lógica de null no frontend.
