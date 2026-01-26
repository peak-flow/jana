#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;

use db::DbState;

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let db_state = rt.block_on(async {
        DbState::init().await.expect("Failed to initialize database")
    });

    tauri::Builder::default()
        .manage(db_state)
        .invoke_handler(tauri::generate_handler![
            commands::notes::create_note,
            commands::notes::save_note,
            commands::notes::get_note,
            commands::notes::list_notes,
            commands::notes::delete_note,
            commands::llm::summarize_note,
            commands::llm::get_summary,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
