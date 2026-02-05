#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod frontmatter;

use db::DbState;

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let db_state = rt.block_on(async {
        DbState::init().await.expect("Failed to initialize database")
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(db_state)
        .invoke_handler(tauri::generate_handler![
            commands::files::create_new_file,
            commands::files::open_file_dialog,
            commands::files::read_file,
            commands::files::save_file,
            commands::files::save_file_as,
            commands::files::save_temp_file_as,
            commands::files::close_file,
            commands::files::list_open_files,
            commands::files::update_cursor_position,
            commands::files::fork_file,
            commands::files::clear_ai_history,
            commands::files::reveal_in_finder,
            commands::llm::summarize_file,
            commands::llm::get_file_summary,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
