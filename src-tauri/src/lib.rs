#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub mod ai;
pub mod commands;
pub mod git;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
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
            commands::git::push_git_changes,
            commands::git::pull_git_changes,
            commands::git::configure_git_origin,
            commands::git::repair_git_upstream,
            commands::git::open_git_file_external,
            commands::git::mark_git_files_resolved,
            commands::git::abort_git_merge,
            commands::git::load_git_log,
            commands::git::load_git_commit_detail,
            commands::git::load_git_commit_file_diff,
            commands::git::ensure_git_project_profile,
            commands::git::load_git_project_profile,
            commands::git::save_git_project_profile,
            commands::git::build_git_commit_prompt,
            commands::git::generate_git_ai_analysis,
            commands::git::generate_git_ai_analysis_from_prompt,
            commands::git::test_ai_model_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

