use tauri::menu::{AboutMetadata, Menu, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Emitter, Manager, Wry};

/// Custom menu item ids relayed to the frontend. Predefined items (clipboard,
/// undo/redo, window ops, quit) act natively on the focused webview and are not
/// listed here, so the event handler ignores them.
const CUSTOM_IDS: &[&str] = &[
    "settings",
    "new-file",
    "new-window",
    "open-file",
    "save",
    "save-as",
    "reveal",
    "close-tab",
    "toggle-panel",
];

/// Build the application menu bar. Custom items carry stable ids that the menu
/// event handler relays to the focused window; predefined items act natively.
pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let settings = MenuItemBuilder::with_id("settings", "Settings…")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let app_menu = SubmenuBuilder::new(app, "Jana")
        .about(Some(AboutMetadata::default()))
        .separator()
        .item(&settings)
        .separator()
        .hide()
        .quit()
        .build()?;

    let new_file = MenuItemBuilder::with_id("new-file", "New File")
        .accelerator("CmdOrCtrl+N")
        .build(app)?;
    let new_window = MenuItemBuilder::with_id("new-window", "New Window")
        .accelerator("CmdOrCtrl+Shift+N")
        .build(app)?;
    let open_file = MenuItemBuilder::with_id("open-file", "Open…")
        .accelerator("CmdOrCtrl+O")
        .build(app)?;
    let save = MenuItemBuilder::with_id("save", "Save")
        .accelerator("CmdOrCtrl+S")
        .build(app)?;
    let save_as = MenuItemBuilder::with_id("save-as", "Save As…")
        .accelerator("CmdOrCtrl+Shift+S")
        .build(app)?;
    let reveal = MenuItemBuilder::with_id("reveal", "Reveal in Finder").build(app)?;
    let close_tab = MenuItemBuilder::with_id("close-tab", "Close Tab")
        .accelerator("CmdOrCtrl+W")
        .build(app)?;
    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&new_file)
        .item(&new_window)
        .item(&open_file)
        .separator()
        .item(&save)
        .item(&save_as)
        .separator()
        .item(&reveal)
        .separator()
        .item(&close_tab)
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let toggle_panel = MenuItemBuilder::with_id("toggle-panel", "Toggle AI Panel")
        .accelerator("CmdOrCtrl+Shift+A")
        .build(app)?;
    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&toggle_panel)
        .build()?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .separator()
        .close_window()
        .build()?;

    Menu::with_items(
        app,
        &[&app_menu, &file_menu, &edit_menu, &view_menu, &window_menu],
    )
}

/// Relay a custom menu selection to the focused window's frontend. The frontend
/// listens for `menu-action` and runs the matching handler. Predefined item ids
/// are ignored here (the OS already performed the action).
pub fn handle_event(app: &AppHandle, id: &str) {
    if !CUSTOM_IDS.contains(&id) {
        return;
    }
    if let Some(win) = app
        .webview_windows()
        .into_values()
        .find(|w| w.is_focused().unwrap_or(false))
    {
        let _ = win.emit("menu-action", id);
    }
}
