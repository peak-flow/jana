#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod buffers;
mod commands;
mod db;
mod frontmatter;

use db::DbState;
use tauri::Manager;

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let db_state = rt.block_on(async {
        DbState::init().await.expect("Failed to initialize database")
    });

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(db_state)
        .manage(buffers::BufferRegistry::default())
        .setup(|app| {
            // Single disk writer: periodically flush dirty, idle buffers to disk.
            let registry = app.state::<buffers::BufferRegistry>().inner().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                buffers::flush_idle_buffers(&registry);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::files::create_new_file,
            commands::files::open_file_dialog,
            commands::files::add_tab,
            commands::files::create_app_window,
            commands::files::save_temp_file_as,
            commands::files::close_tab,
            commands::files::list_tabs,
            commands::files::update_tab_view,
            commands::files::fork_file,
            commands::files::clear_ai_history,
            commands::files::reveal_in_finder,
            buffers::acquire_buffer,
            buffers::get_buffer,
            buffers::update_buffer,
            buffers::release_buffer,
            commands::llm::summarize_file,
            commands::llm::get_file_summary,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        // Flush any unsaved buffers before the process exits.
        if let tauri::RunEvent::ExitRequested { .. } = event {
            let registry = app_handle.state::<buffers::BufferRegistry>();
            buffers::flush_all(&registry);
        }
    });
}
