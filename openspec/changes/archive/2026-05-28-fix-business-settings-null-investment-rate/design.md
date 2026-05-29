## Context

O banco tem `investment_rate DECIMAL(5,4) NOT NULL DEFAULT 0` e `profit_margin DECIMAL(5,4) NOT NULL DEFAULT 0`. O comando Rust espera `f64`. O frontend converte strings com `parseFloat`, que retorna `NaN` para campos vazios; `NaN` serializado via Tauri IPC vira `null` no JSON; Rust rejeita `null` para `f64`.

Há dois locais que escrevem nos campos (`create_business_settings` em auth.rs e `update_business_settings` em settings.rs) e dois que leem (`get_business_info` em settings.rs e `get_pricing_rates` em products.rs).

## Goals / Non-Goals

**Goals:**
- Permitir salvar empresa com campos de taxa vazios (null no banco).
- Distinguir semanticamente "não configurado" de "configurado como 0%".
- Exibir campo vazio na UI quando o valor é null.

**Non-Goals:**
- Bloquear cálculo de precificação quando taxas são null (tratar como 0.0 é suficiente).
- Alterar `PricingRates` (struct retornada por `get_pricing_rates`) — permanece `f64`; null é resolvido internamente.

## Decisions

**1. Schema: remover NOT NULL, manter DEFAULT**

```sql
-- Antes
"investment_rate" DECIMAL(5,4) NOT NULL DEFAULT 0,
-- Depois
"investment_rate" DECIMAL(5,4),
```

Remover o `DEFAULT 0` junto com NOT NULL: se o usuário não passar o campo, o banco armazena NULL explicitamente. Isso é o comportamento desejado.

**2. Migração para DBs existentes via ALTER COLUMN**

DuckDB suporta `ALTER TABLE ... ALTER COLUMN ... DROP NOT NULL`. Adicionar função `ensure_business_settings_nullable` em db.rs, chamada no startup junto com as demais migrations.

**3. Rust: `Option<f64>` para input e output de business settings**

- `create_business_settings(investment_rate: Option<f64>, ...)`: validar range apenas se `Some`.
- `update_business_settings(investment_rate: Option<f64>, ...)`: idem.
- `BusinessInfo { investment_rate: Option<f64>, ... }`: serializa como `number | null` para o frontend.
- `get_pricing_rates`: lê `Option<f64>` do banco, usa `.unwrap_or(0.0)` antes de retornar `PricingRates` (que permanece `f64`).

**4. Frontend: null explícito quando campo vazio**

```ts
// Antes
investmentRate: parseFloat(form.investmentRate) / 100,

// Depois
investmentRate: form.investmentRate !== "" ? parseFloat(form.investmentRate) / 100 : null,
```

Ao carregar, null → string vazia no estado do formulário.

## Risks / Trade-offs

- [Migração DROP NOT NULL] DuckDB pode se comportar diferente em colunas com constraint composta. Mitigação: testar com `test_db()` no Rust antes de fazer rollout.
- [Pricing com null] Tratar null como 0.0 no markup pode gerar precificação incorreta silenciosamente se o usuário esquecer de configurar as taxas. Trade-off aceito: a UI de configurações mostra os campos em branco, servindo de lembrete visual.
- [EmpresaSection] `info.investment_rate` pode ser null; `String(Math.round(null * 10000) / 100)` retornaria `"0"`. Precisa tratar: `info.investment_rate != null ? ... : ""`.
