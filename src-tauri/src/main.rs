// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod enums;
mod utils;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_store::StoreExt;
use tokio::time::{self, Duration};

use crate::utils::{check_minecraft_running, log_to_file_and_emit};

#[tokio::main]
async fn main() {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let configure_i = MenuItem::with_id(
                app,
                "configure",
                "Configurar destino do backup",
                true,
                None::<&str>,
            )?;

            let menu = Menu::with_items(app, &[&configure_i, &quit_i])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "configure" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    _ => log_to_file_and_emit(
                        &app.app_handle().clone(),
                        format!("menu item {:?} not handled", event.id),
                    ),
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            let app_handle_15_secs = app.handle().clone();

            let store = app.store("settings.json")?;
            let store_15_secs = store.clone();

            let time_to_backup = store.get("time_to_backup").unwrap_or_default();

            tauri::async_runtime::spawn(async move {
                let mut interval = time::interval(Duration::from_secs(15));
                loop {
                    interval.tick().await;
                    let status = check_minecraft_running(&app_handle_15_secs).await;
                    match status {
                        enums::WorldStatus::InWorld(world_name) => {
                            log_to_file_and_emit(
                                &app_handle_15_secs,
                                format!(
                                    "[FAST CHECK] Minecraft World: {}. Setando como último mundo para backup.",
                                    world_name
                                ),
                            );
                            store_15_secs
                                .set("last_backed_up_world", world_name);
                                
                        }
                        enums::WorldStatus::Stopped => {
                            let last_world = store_15_secs
                                .get("last_backed_up_world")
                                .unwrap_or_default();

                            if last_world.is_null() {
                                log_to_file_and_emit(
                                    &app_handle_15_secs,
                                    "[FAST CHECK] Sem backup pendente."
                                        .to_string(),
                                );
                            } else {
                                log_to_file_and_emit(
                                    &app_handle_15_secs,
                                    format!(
                                        "[FAST CHECK] Minecraft World: {}. Iniciando backup...",
                                        last_world
                                    ),
                                );
                                utils::backup_minecraft_world(
                                    &app_handle_15_secs,
                                    &store_15_secs,
                                    last_world.as_str().unwrap().to_string(),
                                )
                                .await;
                                store_15_secs.delete("last_backed_up_world");
                            }
                        }
                        _ => {}
                    }
                }
            });

            tauri::async_runtime::spawn(async move {
                if time_to_backup.is_null() {
                    log_to_file_and_emit(
                        &app_handle,
                        "[BACKGROUND TASK] time_to_backup não está configurado corretamente.",
                    );
                    return;
                }
                let interval_secs = time_to_backup.as_i64().unwrap() as u64 * 60;
                let mut interval = time::interval(Duration::from_secs(interval_secs));

                loop {
                    interval.tick().await;
                    let status = check_minecraft_running(&app_handle).await;
                    log_to_file_and_emit(
                        &app_handle,
                        format!("[BACKGROUND TASK] Minecraft status: {:?}", status),
                    );

                    match status {
                        enums::WorldStatus::InWorld(world_name) => {
                            log_to_file_and_emit(
                                &app_handle,
                                format!(
                                    "[BACKGROUND TASK] Minecraft World: {}. Iniciando backup...",
                                    world_name
                                ),
                            );
                            utils::backup_minecraft_world(&app_handle, &store, world_name).await;
                        }
                        _ => {}
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|app_handle, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                app_handle.hide().unwrap();
            }
            _ => {}
        })
        .run(context)
        .expect("failed to run app");
}
