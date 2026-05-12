// src-tauri/src/commands/migration.rs
//
// One-shot IPC to pull (and clear) the migration report cached at app startup.
// Using IPC pull instead of event push avoids the setup-vs-mount race: Tauri's
// `setup` closure runs before the WebView window exists, so events emitted there
// are lost by the time the frontend's `onMount` registers a listener.

use crate::app_config::MigrationReport;
use crate::state::AppState;
use tauri::State;

/// Returns and clears the one-shot migration report cached at app startup.
/// Subsequent calls return `null` (None).
#[tauri::command]
pub async fn take_migration_report(
    state: State<'_, AppState>,
) -> Result<Option<MigrationReport>, String> {
    Ok(state.take_migration_report().await)
}
