#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use tauri::menu::{Menu, MenuItem, Submenu};

fn main() {
    tauri::Builder::default()
        .menu(|app| {
            let menu = Menu::new(app)?;
            let groups: &[(&str, &[&str])] = &[
                #[cfg(target_os = "macos")]
                ("Text Editor", &["Settings..."]),
                ("File", &["New File", "Open File...", "Save", "Save As..."]),
                ("Edit", &["Undo", "Redo", "Cut", "Copy", "Paste", "Find..."]),
                ("View", &["Zoom In", "Zoom Out", "Toggle Word Wrap"]),
                ("Help", &["About Text Editor"]),
            ];

            for &(title, items) in groups {
                let submenu = Submenu::new(app, title, true)?;
                for &label in items {
                    submenu.append(&MenuItem::new(app, label, false, None::<&str>)?)?;
                }
                menu.append(&submenu)?;
            }

            Ok(menu)
        })
        .run(tauri::generate_context!())
        .expect("failed to run text editor");
}
