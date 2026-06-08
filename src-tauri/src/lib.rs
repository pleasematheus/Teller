use std::sync::Mutex;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

pub mod auth;
pub mod db;
pub mod financials;
pub mod products;
pub mod reports;
pub mod sales;
pub mod settings;
pub mod stock;

pub use db::{DbState, init_schema, open_database, run_migrations};

fn resolve_db_path(
    app_env: Option<&str>,
    dev: Option<&str>,
    prod: Option<&str>,
    data_dir: &std::path::Path,
) -> std::path::PathBuf {
    match app_env {
        Some("development") => {
            let p = dev.unwrap_or("").trim();
            if p.is_empty() {
                std::path::PathBuf::from("./teller-dev.db")
            } else {
                std::path::PathBuf::from(p)
            }
        }
        _ => {
            let p = prod.unwrap_or("").trim();
            if p.is_empty() {
                data_dir.join("teller.db")
            } else {
                std::path::PathBuf::from(p)
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_env = std::env::var("APP_ENV").ok();
            let db_path_dev = std::env::var("DB_PATH_DEV").ok();
            let db_path_prod = std::env::var("DB_PATH_PROD").ok();
            let data_dir = app.path().app_data_dir()?;
            let db_path = resolve_db_path(
                app_env.as_deref(),
                db_path_dev.as_deref(),
                db_path_prod.as_deref(),
                &data_dir,
            );
            if db_path.starts_with(&data_dir) {
                std::fs::create_dir_all(&data_dir)?;
            }
            let io_err = |e: String| std::io::Error::new(std::io::ErrorKind::Other, e);
            let conn = open_database(&db_path)
                .map_err(|e| io_err(e.to_string()))?;
            init_schema(&conn).map_err(&io_err)?;
            drop(conn);
            run_migrations(Some(&db_path)).map_err(&io_err)?;
            let conn = open_database(&db_path)
                .map_err(|e| io_err(e.to_string()))?;
            app.manage(DbState(Mutex::new(conn)));

            let show_item = MenuItem::with_id(app, "show", "Abrir", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Teller")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        if let Ok(conn) = app.state::<DbState>().0.lock() {
                            let _ = conn.execute_batch("CHECKPOINT");
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

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
            stock::close_purchase,
            stock::get_purchase_draft,
            stock::update_purchase,
            stock::delete_purchase,
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
            sales::get_sale_payments,
            sales::get_product_sales_history,
            stock::list_product_purchases,
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
            settings::rename_payment_method,
            settings::delete_payment_method,
            settings::list_transaction_fees,
            settings::upsert_transaction_fee,
            settings::list_units_of_measure,
            settings::create_unit_of_measure,
            settings::rename_unit_of_measure,
            settings::update_unit_of_measure,
            settings::delete_unit_of_measure,
            settings::get_system_setting,
            settings::set_system_setting,
            reports::get_dre,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            match event {
                tauri::RunEvent::WindowEvent {
                    label,
                    event: tauri::WindowEvent::CloseRequested { api, .. },
                    ..
                } => {
                    let minimize = app_handle
                        .state::<DbState>()
                        .0
                        .lock()
                        .ok()
                        .and_then(|conn| {
                            conn.query_row(
                                r#"SELECT COALESCE((SELECT "value" FROM "System_Settings" WHERE "key" = 'minimize_to_tray'), 'false')"#,
                                [],
                                |r| r.get::<_, String>(0),
                            ).ok()
                        })
                        .map(|v| v == "true")
                        .unwrap_or(false);

                    if minimize {
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window(&label) {
                            let _ = window.hide();
                        }
                    }
                }
                tauri::RunEvent::WindowEvent {
                    event: tauri::WindowEvent::Destroyed,
                    ..
                } => {
                    if let Ok(conn) = app_handle.state::<DbState>().0.lock() {
                        let _ = conn.execute_batch("CHECKPOINT");
                    }
                }
                _ => {}
            }
        });
}

#[cfg(test)]
mod resolve_db_path_tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn dev_custom_path_returned() {
        let data_dir = Path::new("/data");
        let result = resolve_db_path(Some("development"), Some("./custom-dev.db"), None, data_dir);
        assert_eq!(result, std::path::PathBuf::from("./custom-dev.db"));
    }

    #[test]
    fn dev_absent_path_uses_default() {
        let data_dir = Path::new("/data");
        let result = resolve_db_path(Some("development"), None, None, data_dir);
        assert_eq!(result, std::path::PathBuf::from("./teller-dev.db"));
    }

    #[test]
    fn dev_empty_path_uses_default() {
        let data_dir = Path::new("/data");
        let result = resolve_db_path(Some("development"), Some("  "), None, data_dir);
        assert_eq!(result, std::path::PathBuf::from("./teller-dev.db"));
    }

    #[test]
    fn production_custom_path_returned() {
        let data_dir = Path::new("/data");
        let result = resolve_db_path(Some("production"), None, Some("D:/dados/teller.db"), data_dir);
        assert_eq!(result, std::path::PathBuf::from("D:/dados/teller.db"));
    }

    #[test]
    fn production_absent_path_uses_data_dir() {
        let data_dir = Path::new("/data");
        let result = resolve_db_path(Some("production"), None, None, data_dir);
        assert_eq!(result, std::path::PathBuf::from("/data/teller.db"));
    }

    #[test]
    fn absent_app_env_falls_to_production() {
        let data_dir = Path::new("/data");
        let result = resolve_db_path(None, None, None, data_dir);
        assert_eq!(result, std::path::PathBuf::from("/data/teller.db"));
    }

    #[test]
    fn unknown_app_env_falls_to_production() {
        let data_dir = Path::new("/data");
        let result = resolve_db_path(Some("staging"), None, None, data_dir);
        assert_eq!(result, std::path::PathBuf::from("/data/teller.db"));
    }
}
