//! MyLife backend. The UI calls the `#[tauri::command]` functions below.
//!
//! The database stays locked (no connection) until the user enters the
//! app password. Every command that reads data fails with "Locked." until then.

pub mod agent;
pub mod db;
pub mod integrations;
pub mod modules;
pub mod rules;
pub mod scheduler;
pub mod vault;

use std::sync::Mutex;

use chrono::{NaiveDate, Utc};
use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use zeroize::Zeroizing;

use db::{DbError, DbPaths};
use modules::calendar::{self, Event, EventInput, Occurrence};
use modules::tasks::{self, NewTask, Task, TaskPatch, TaskView};
use modules::today::{self, Today};
use rules::store::SavedRule;
use scheduler::reminders::{self, Reminder, Snooze};

pub struct AppState {
    pub(crate) paths: DbPaths,
    pub(crate) conn: Mutex<Option<Connection>>,
}

/// Commands return plain text errors the UI can show as-is.
type CmdResult<T> = Result<T, String>;

fn err(e: DbError) -> String {
    e.to_string()
}

/// Runs `f` with the open database, or fails with "Locked." if it is locked.
fn with_db<T>(state: &State<AppState>, f: impl FnOnce(&Connection) -> Result<T, DbError>) -> CmdResult<T> {
    let guard = state.conn.lock().map_err(|_| "App state is broken. Restart MyLife.".to_string())?;
    match guard.as_ref() {
        Some(c) => f(c).map_err(err),
        None => Err("Locked.".into()),
    }
}

#[derive(Serialize)]
struct AppStatus {
    /// False on first run, before an app password exists.
    created: bool,
    unlocked: bool,
}

#[tauri::command]
fn app_status(state: State<AppState>) -> CmdResult<AppStatus> {
    let unlocked = state.conn.lock().map(|c| c.is_some()).unwrap_or(false);
    Ok(AppStatus { created: state.paths.exists(), unlocked })
}

/// After the database opens: add starter rules, then run the agent once
/// so anything missed while locked shows right away.
fn opened(app: &AppHandle, state: &State<AppState>, conn: Connection) -> CmdResult<()> {
    rules::store::seed_starters(&conn).map_err(err)?;
    *state.conn.lock().unwrap() = Some(conn);
    agent::tick(app);
    Ok(())
}

#[tauri::command]
fn create_password(app: AppHandle, state: State<AppState>, password: String) -> CmdResult<()> {
    let password = Zeroizing::new(password);
    let conn = db::create(&state.paths, &password).map_err(err)?;
    opened(&app, &state, conn)
}

#[tauri::command]
fn unlock(app: AppHandle, state: State<AppState>, password: String) -> CmdResult<()> {
    let password = Zeroizing::new(password);
    let conn = db::unlock(&state.paths, &password).map_err(err)?;
    opened(&app, &state, conn)
}

#[tauri::command]
fn lock(state: State<AppState>) -> CmdResult<()> {
    // Dropping the connection closes the file. The key is gone from memory.
    *state.conn.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
fn today(state: State<AppState>) -> CmdResult<Today> {
    with_db(&state, |c| today::build(c, Utc::now()))
}

#[tauri::command]
fn tasks_list(state: State<AppState>, view: TaskView) -> CmdResult<Vec<Task>> {
    with_db(&state, |c| tasks::list(c, view, Utc::now()))
}

#[tauri::command]
fn task_create(state: State<AppState>, task: NewTask) -> CmdResult<Task> {
    with_db(&state, |c| tasks::create(c, &task))
}

#[tauri::command]
fn task_update(state: State<AppState>, id: String, patch: TaskPatch) -> CmdResult<Task> {
    with_db(&state, |c| tasks::update(c, &id, &patch))
}

#[tauri::command]
fn task_set_done(state: State<AppState>, id: String, done: bool) -> CmdResult<Task> {
    with_db(&state, |c| tasks::set_done(c, &id, done))
}

#[tauri::command]
fn task_archive(state: State<AppState>, id: String) -> CmdResult<()> {
    with_db(&state, |c| tasks::archive(c, &id))
}

#[tauri::command]
fn task_add_steps(state: State<AppState>, id: String, titles: Vec<String>) -> CmdResult<Vec<Task>> {
    with_db(&state, |c| tasks::add_steps(c, &id, &titles))
}

#[tauri::command]
fn task_steps(state: State<AppState>, id: String) -> CmdResult<Vec<Task>> {
    with_db(&state, |c| tasks::steps(c, &id))
}

/// Moves a task to a new day. With no date, moves it to today.
#[tauri::command]
fn task_reschedule(state: State<AppState>, id: String, date: Option<String>) -> CmdResult<Task> {
    let day = match date {
        Some(d) => NaiveDate::parse_from_str(&d, "%Y-%m-%d").map_err(|_| "Date must be YYYY-MM-DD.".to_string())?,
        None => db::ids::local_today(Utc::now()),
    };
    with_db(&state, |c| tasks::reschedule(c, &id, day))
}

/// All event occurrences that overlap `[from, to)`. Times are UTC ISO strings.
#[tauri::command]
fn events_range(state: State<AppState>, from: String, to: String) -> CmdResult<Vec<Occurrence>> {
    let parse = |s: &str| {
        chrono::DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&Utc)).map_err(|_| format!("Bad time: {s}"))
    };
    let (from, to) = (parse(&from)?, parse(&to)?);
    with_db(&state, |c| calendar::occurrences(c, from, to))
}

#[tauri::command]
fn event_get(state: State<AppState>, id: String) -> CmdResult<Event> {
    with_db(&state, |c| calendar::get(c, &id))
}

#[tauri::command]
fn event_create(state: State<AppState>, event: EventInput) -> CmdResult<Event> {
    with_db(&state, |c| calendar::create(c, &event))
}

#[tauri::command]
fn event_update(state: State<AppState>, id: String, event: EventInput) -> CmdResult<Event> {
    with_db(&state, |c| calendar::update(c, &id, &event))
}

#[tauri::command]
fn event_archive(state: State<AppState>, id: String) -> CmdResult<()> {
    with_db(&state, |c| calendar::archive(c, &id))
}

/// Drag a task onto the calendar: makes a time block for it.
#[tauri::command]
fn task_block(state: State<AppState>, id: String, starts_at: String) -> CmdResult<Event> {
    with_db(&state, |c| calendar::block_task(c, &id, &starts_at))
}

#[tauri::command]
fn reminders_active(state: State<AppState>) -> CmdResult<Vec<Reminder>> {
    with_db(&state, |c| reminders::active(c, Utc::now()))
}

/// Marks a reminder done. If it belongs to a task, the task stays as is.
#[tauri::command]
fn reminder_done(state: State<AppState>, id: String) -> CmdResult<()> {
    with_db(&state, |c| reminders::set_state(c, &id, "done"))
}

#[tauri::command]
fn reminder_snooze(state: State<AppState>, id: String, option: Snooze) -> CmdResult<Reminder> {
    with_db(&state, |c| reminders::snooze(c, &id, option, Utc::now()))
}

#[tauri::command]
fn rules_list(state: State<AppState>) -> CmdResult<Vec<SavedRule>> {
    with_db(&state, rules::store::list)
}

#[tauri::command]
fn rule_create(app: AppHandle, state: State<AppState>, rule: rules::Rule) -> CmdResult<SavedRule> {
    let saved = with_db(&state, |c| rules::store::create(c, "tasks", &rule))?;
    agent::tick(&app);
    Ok(saved)
}

#[tauri::command]
fn rule_set_enabled(state: State<AppState>, id: String, enabled: bool) -> CmdResult<SavedRule> {
    with_db(&state, |c| rules::store::set_enabled(c, &id, enabled))
}

#[tauri::command]
fn rule_delete(state: State<AppState>, id: String) -> CmdResult<()> {
    with_db(&state, |c| rules::store::delete(c, &id))
}

/// Spec: Ctrl+Shift+Space opens quick capture from any app.
fn quick_capture_shortcut() -> Shortcut {
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed && shortcut == &quick_capture_shortcut() {
                        agent::open_quick_capture(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            app.manage(AppState { paths: DbPaths::in_dir(&dir), conn: Mutex::new(None) });
            agent::setup_tray(app.handle())?;
            agent::start(app.handle().clone());
            // Spec: the agent starts at Windows login. Only for installed builds.
            #[cfg(not(debug_assertions))]
            {
                use tauri_plugin_autostart::ManagerExt;
                let _ = app.autolaunch().enable();
            }
            // If another app already owns the hotkey, keep running without it.
            if let Err(e) = app.global_shortcut().register(quick_capture_shortcut()) {
                eprintln!("Quick capture hotkey not available: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_status,
            create_password,
            unlock,
            lock,
            today,
            tasks_list,
            task_create,
            task_update,
            task_set_done,
            task_archive,
            task_add_steps,
            task_steps,
            task_reschedule,
            events_range,
            event_get,
            event_create,
            event_update,
            event_archive,
            task_block,
            reminders_active,
            reminder_done,
            reminder_snooze,
            rules_list,
            rule_create,
            rule_set_enabled,
            rule_delete,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MyLife");
}
