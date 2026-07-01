#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub mod ai;
pub mod commands;
pub mod git;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .manage(commands::project_process::ProjectProcessState::default())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                window.set_icon(tauri::include_image!("icons/window-icon.png"))?;
            }

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::git::load_git_snapshot,
            commands::git::load_git_file_diff,
            commands::git::load_git_file_head_diff,
            commands::git::commit_git_changes,
            commands::git::fetch_git_changes,
            commands::git::sync_git_status,
            commands::git::push_git_changes,
            commands::git::pull_git_changes,
            commands::git::rebase_git_changes,
            commands::git::configure_git_origin,
            commands::git::repair_git_upstream,
            commands::git::open_git_file_external,
            commands::git::mark_git_files_resolved,
            commands::git::abort_git_merge,
            commands::git::continue_git_merge,
            commands::git::continue_git_rebase,
            commands::git::abort_git_rebase,
            commands::git::load_git_log,
            commands::git::load_git_commit_detail,
            commands::git::load_git_commit_file_diff,
            commands::git::ensure_git_project_profile,
            commands::git::load_git_project_profile,
            commands::git::save_git_project_profile,
            commands::git::build_git_commit_prompt,
            commands::git::generate_git_ai_analysis,
            commands::git::generate_git_ai_analysis_from_prompt,
            commands::git::test_ai_model_connection,
            commands::ai_settings::load_ai_settings,
            commands::ai_settings::save_ai_settings,
            commands::project::load_project_manifest,
            commands::project_process::start_project_process,
            commands::project_process::list_project_processes,
            commands::project_process::stop_project_process,
            commands::project_process::restart_project_process,
            commands::project_process::load_project_process_logs,
            commands::project_process::open_project_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

