use duckdb::params;
use serde::Serialize;
use tauri::State;

use crate::auth::{require_session, SessionState};
use crate::db::DbState;

#[derive(Serialize)]
pub struct DreLineItem {
    pub category: String,
    pub subtype: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct DreResult {
    pub receita_bruta: f64,
    pub receita_liquida: f64,
    pub lucro_bruto: f64,
    pub lucro_operacional: f64,
    pub ebit: f64,
    pub lucro_liquido: f64,
    pub cmv_partial: bool,
    pub line_items: Vec<DreLineItem>,
}

#[tauri::command]
pub fn get_dre(
    start_date: String,
    end_date: String,
    session_token: String,
    state: State<DbState>,
    session: State<SessionState>,
) -> Result<DreResult, String> {
    require_session(&session, &session_token)?;
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    compute_dre(&conn, &start_date, &end_date)
}

pub fn compute_dre(
    conn: &duckdb::Connection,
    start_date: &str,
    end_date: &str,
) -> Result<DreResult, String> {
    // Receita de vendas: preço cheio × quantidade, vendas fechadas no período
    let receita_vendas: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(sp.price * sp.quantity), 0.0)
               FROM "Sales_Products" sp
               JOIN "Sales" s ON s.id = sp.sale_id
               WHERE s.status = 'closed'
                 AND CAST(s.updated_at AS DATE) BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Receita financeira: Financial_Entries income no período
    let receita_financeira: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(amount), 0.0)
               FROM "Financial_Entries"
               WHERE type = 'income'
                 AND reference_date BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let receita_bruta = receita_vendas + receita_financeira;

    // Descontos concedidos nas vendas
    let descontos: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(sp.price * sp.quantity - sp.discounted_price * sp.quantity), 0.0)
               FROM "Sales_Products" sp
               JOIN "Sales" s ON s.id = sp.sale_id
               WHERE s.status = 'closed'
                 AND CAST(s.updated_at AS DATE) BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Taxas de pagamento retidas (amount_paid - net_amount)
    let taxas: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(sp.amount_paid - sp.net_amount), 0.0)
               FROM "Sale_Payments" sp
               JOIN "Sales" s ON s.id = sp.sale_id
               WHERE s.status = 'closed'
                 AND CAST(s.updated_at AS DATE) BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Devoluções: Financial_Entries expense_subtype = 'devolucao'
    let devolucoes: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(amount), 0.0)
               FROM "Financial_Entries"
               WHERE type = 'expense'
                 AND expense_subtype = 'devolucao'
                 AND reference_date BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let receita_liquida = receita_bruta - descontos - taxas - devolucoes;

    // CMV: custo dos produtos vendidos (apenas itens com cost_price snapshot)
    let cmv: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(sp.cost_price * sp.quantity), 0.0)
               FROM "Sales_Products" sp
               JOIN "Sales" s ON s.id = sp.sale_id
               WHERE s.status = 'closed'
                 AND sp.cost_price IS NOT NULL
                 AND CAST(s.updated_at AS DATE) BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Verifica se há itens sem cost_price (CMV parcial)
    let items_sem_custo: i64 = conn
        .query_row(
            r#"SELECT COUNT(*)
               FROM "Sales_Products" sp
               JOIN "Sales" s ON s.id = sp.sale_id
               WHERE s.status = 'closed'
                 AND sp.cost_price IS NULL
                 AND CAST(s.updated_at AS DATE) BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let lucro_bruto = receita_liquida - cmv;

    // Despesas operacionais: expense_subtype IN ('despesa', 'custo')
    let desp_operacional: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(amount), 0.0)
               FROM "Financial_Entries"
               WHERE type = 'expense'
                 AND expense_subtype IN ('despesa', 'custo')
                 AND reference_date BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let lucro_operacional = lucro_bruto - desp_operacional;

    // Juros
    let juros: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(amount), 0.0)
               FROM "Financial_Entries"
               WHERE type = 'expense'
                 AND expense_subtype = 'juros'
                 AND reference_date BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let ebit = lucro_operacional - juros;

    // Impostos: IR, CSLL
    let impostos: f64 = conn
        .query_row(
            r#"SELECT COALESCE(SUM(amount), 0.0)
               FROM "Financial_Entries"
               WHERE type = 'expense'
                 AND expense_subtype = 'imposto'
                 AND reference_date BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)"#,
            params![start_date, end_date],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    let lucro_liquido = ebit - impostos;

    // Line items por categoria e subtipo (para o Sankey)
    let mut line_stmt = conn
        .prepare(
            r#"SELECT LOWER(TRIM(COALESCE(expense_subtype, 'outro'))),
                      LOWER(TRIM(category)),
                      SUM(amount)
               FROM "Financial_Entries"
               WHERE type = 'expense'
                 AND expense_subtype IS NOT NULL
                 AND reference_date BETWEEN CAST(? AS DATE) AND CAST(? AS DATE)
               GROUP BY LOWER(TRIM(COALESCE(expense_subtype, 'outro'))), LOWER(TRIM(category))
               ORDER BY LOWER(TRIM(COALESCE(expense_subtype, 'outro'))), SUM(amount) DESC"#,
        )
        .map_err(|e| e.to_string())?;

    let line_items = line_stmt
        .query_map(params![start_date, end_date], |row| {
            Ok(DreLineItem {
                subtype: row.get(0)?,
                category: row.get(1)?,
                amount: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(DreResult {
        receita_bruta,
        receita_liquida,
        lucro_bruto,
        lucro_operacional,
        ebit,
        lucro_liquido,
        cmv_partial: items_sem_custo > 0,
        line_items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::insert_user;
    use crate::db::test_db;
    use crate::products::insert_product;
    use crate::sales::{do_close_sale_work, SaleItemInput, SalePaymentInput};
    use crate::settings::insert_payment_method;
    use duckdb::params;

    fn insert_financial_entry(
        conn: &duckdb::Connection,
        entry_type: &str,
        category: &str,
        subtype: Option<&str>,
        amount: f64,
        date: &str,
    ) {
        conn.execute(
            r#"INSERT INTO "Financial_Entries"
               (type, name, category, amount, expense_subtype, is_fixed, reference_date, updated_at, created_at)
               VALUES (?, ?, ?, ?, ?, false, CAST(? AS DATE), CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            params![entry_type, category, category, amount, subtype, date],
        )
        .unwrap();
    }

    #[test]
    fn dre_periodo_sem_dados_retorna_zeros() {
        let conn = test_db();
        let result = compute_dre(&conn, "2099-01-01", "2099-01-31").unwrap();
        assert_eq!(result.receita_bruta, 0.0);
        assert_eq!(result.receita_liquida, 0.0);
        assert_eq!(result.lucro_bruto, 0.0);
        assert_eq!(result.lucro_operacional, 0.0);
        assert_eq!(result.ebit, 0.0);
        assert_eq!(result.lucro_liquido, 0.0);
        assert!(!result.cmv_partial);
    }

    #[test]
    fn dre_periodo_com_vendas_e_despesas_retorna_indicadores_corretos() {
        let conn = test_db();
        insert_user(&conn, "u", "u", true);
        let user_id: i64 = conn
            .query_row(r#"SELECT id FROM "Users" LIMIT 1"#, [], |r| r.get(0))
            .unwrap();
        let pm_id = insert_payment_method(&conn, "Dinheiro");
        let product_id = insert_product(&conn, "Foto", true);
        conn.execute(
            r#"UPDATE "Products" SET cost_price = 5.0 WHERE id = ?"#,
            params![product_id],
        )
        .unwrap();

        let sale_id: i64 = conn
            .query_row(
                r#"INSERT INTO "Sales" (user_id, label, status, total_value, updated_at, created_at)
                   VALUES (?, '', 'open', 20.0, '2026-05-10', CURRENT_TIMESTAMP)
                   RETURNING id"#,
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        conn.execute(
            r#"INSERT INTO "Sales_Products" (sale_id, product_id, quantity, price, discount, discounted_price, created_at)
               VALUES (?, ?, 1, 20.0, 0.0, 20.0, CURRENT_TIMESTAMP)"#,
            params![sale_id, product_id],
        )
        .unwrap();

        let items = vec![SaleItemInput {
            product_id,
            quantity: 1,
            discount: 0.0,
        }];
        let payments = vec![SalePaymentInput {
            payment_method_id: pm_id,
            amount_paid: 20.0,
            installments: 1,
        }];
        conn.execute_batch("BEGIN").unwrap();
        do_close_sale_work(&conn, sale_id, &items, &payments).unwrap();
        conn.execute(
            r#"UPDATE "Sales" SET updated_at = '2026-05-10' WHERE id = ?"#,
            params![sale_id],
        )
        .unwrap();
        conn.execute_batch("COMMIT").unwrap();

        insert_financial_entry(
            &conn,
            "expense",
            "aluguel",
            Some("despesa"),
            3.0,
            "2026-05-01",
        );
        insert_financial_entry(&conn, "expense", "banco", Some("juros"), 1.0, "2026-05-01");
        insert_financial_entry(&conn, "expense", "ir", Some("imposto"), 2.0, "2026-05-01");

        let dre = compute_dre(&conn, "2026-05-01", "2026-05-31").unwrap();
        assert!((dre.receita_bruta - 20.0).abs() < 1e-9);
        assert!((dre.receita_liquida - 20.0).abs() < 1e-9);
        assert!((dre.lucro_bruto - 15.0).abs() < 1e-9);
        assert!((dre.lucro_operacional - 12.0).abs() < 1e-9);
        assert!((dre.ebit - 11.0).abs() < 1e-9);
        assert!((dre.lucro_liquido - 9.0).abs() < 1e-9);
    }

    #[test]
    fn commands_reject_invalid_session() {
        use crate::auth::require_session_with;
        let session: Option<crate::auth::Session> = None;
        let err = require_session_with(&session, "bad-token").unwrap_err();
        assert_eq!(err, "Sessão inválida");
    }

    #[test]
    fn dre_com_prejuizo_retorna_valores_negativos() {
        let conn = test_db();
        insert_financial_entry(
            &conn,
            "expense",
            "pessoal",
            Some("custo"),
            1000.0,
            "2026-06-01",
        );
        let dre = compute_dre(&conn, "2026-06-01", "2026-06-30").unwrap();
        assert_eq!(dre.receita_bruta, 0.0);
        assert!(dre.lucro_operacional < 0.0);
        assert!(dre.lucro_liquido < 0.0);
    }
}
