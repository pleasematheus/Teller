use std::sync::Mutex;
use tauri::Manager;

pub mod auth;
pub mod db;
pub mod financials;
pub mod products;
pub mod reports;
pub mod sales;
pub mod settings;
pub mod stock;

pub use db::{DbState, ensure_financial_entries_expense_subtype, ensure_label_column,
             ensure_profit_margin_column, ensure_sales_products_cost_price,
             ensure_unit_column, ensure_user_settings_table, init_schema, open_database};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = match std::env::var("DB_PATH") {
                Ok(p) => std::path::PathBuf::from(p),
                Err(_) => {
                    let data_dir = app.path().app_data_dir()?;
                    std::fs::create_dir_all(&data_dir)?;
                    data_dir.join("teller.db")
                }
            };
            let conn = open_database(&db_path)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            let io_err = |e: String| std::io::Error::new(std::io::ErrorKind::Other, e);
            init_schema(&conn).map_err(&io_err)?;
            ensure_user_settings_table(&conn).map_err(&io_err)?;
            ensure_unit_column(&conn).map_err(&io_err)?;
            ensure_profit_margin_column(&conn).map_err(&io_err)?;
            ensure_label_column(&conn).map_err(&io_err)?;
            ensure_sales_products_cost_price(&conn).map_err(&io_err)?;
            ensure_financial_entries_expense_subtype(&conn).map_err(&io_err)?;
            app.manage(DbState(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::check_setup,
            auth::create_user,
            auth::check_business_setup,
            auth::create_business_settings,
            auth::find_user,
            auth::login,
            auth::get_user_setting,
            auth::set_user_setting,
            products::get_pricing_rates,
            products::get_product_loss_rate,
            products::get_realized_margin,
            products::list_products,
            products::create_product,
            products::update_product,
            stock::list_stock,
            stock::create_purchase,
            stock::create_stock_adjustment,
            stock::list_stock_movements,
            stock::list_product_components,
            stock::add_product_component,
            stock::remove_product_component,
            stock::update_product_component,
            stock::list_purchases,
            sales::list_sales,
            sales::create_sale,
            sales::update_sale,
            sales::close_sale,
            sales::get_sale_items,
            settings::list_payment_methods,
            financials::list_financial_entries,
            financials::list_recurring_suggestions,
            financials::create_financial_entry,
            financials::update_financial_entry,
            financials::delete_financial_entry,
            settings::get_business_info,
            settings::update_business_settings,
            settings::list_users,
            settings::change_user_password,
            settings::toggle_user_active,
            settings::create_payment_method,
            settings::update_payment_method,
            settings::list_transaction_fees,
            settings::upsert_transaction_fee,
            reports::get_dre,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::WindowEvent {
                event: tauri::WindowEvent::Destroyed,
                ..
            } = event
            {
                if let Ok(conn) = app_handle.state::<DbState>().0.lock() {
                    let _ = conn.execute_batch("CHECKPOINT");
                }
            }
        });
}
