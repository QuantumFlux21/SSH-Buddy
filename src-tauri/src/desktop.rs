use std::sync::Mutex;

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Runtime, State, WebviewWindow,
};
use tauri_plugin_autostart::ManagerExt;

use crate::{
    db::Database,
    domain::{AppResult, AppSettings},
};

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_ID: &str = "ssh-buddy-main-tray";
const TRAY_OPEN_ID: &str = "open-ssh-buddy";
const TRAY_QUIT_ID: &str = "quit-ssh-buddy";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBehaviorStatus {
    pub autostart_enabled: Option<bool>,
    pub autostart_error: Option<String>,
    pub tray_available: bool,
    pub tray_error: Option<String>,
}

#[derive(Debug, Clone)]
struct DesktopRuntime {
    close_to_tray_requested: bool,
    tray_available: bool,
    tray_error: Option<String>,
    tray_active: bool,
    explicit_quit: bool,
}

#[derive(Debug)]
pub struct DesktopState {
    runtime: Mutex<DesktopRuntime>,
}

impl DesktopState {
    pub fn new(close_to_tray_requested: bool) -> Self {
        Self {
            runtime: Mutex::new(DesktopRuntime {
                close_to_tray_requested,
                tray_available: false,
                tray_error: None,
                tray_active: false,
                explicit_quit: false,
            }),
        }
    }

    fn snapshot(&self) -> DesktopRuntime {
        self.runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn set_tray_available(&self, active: bool) {
        let mut runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        runtime.tray_available = true;
        runtime.tray_error = None;
        runtime.tray_active = active;
    }

    fn set_tray_error(&self, error: String) {
        let mut runtime = self
            .runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        runtime.tray_available = false;
        runtime.tray_error = Some(error);
        runtime.tray_active = false;
    }

    fn set_close_to_tray_requested(&self, requested: bool) {
        self.runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .close_to_tray_requested = requested;
    }

    fn set_explicit_quit(&self) {
        self.runtime
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .explicit_quit = true;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartupWindowMode {
    Normal,
    Minimized,
}

fn startup_window_mode(database_ready: bool, start_minimized: bool) -> StartupWindowMode {
    if database_ready && start_minimized {
        StartupWindowMode::Minimized
    } else {
        StartupWindowMode::Normal
    }
}

trait WindowOperations {
    fn unminimize(&mut self) -> Result<(), String>;
    fn show(&mut self) -> Result<(), String>;
    fn focus(&mut self) -> Result<(), String>;
    fn minimize(&mut self) -> Result<(), String>;
    fn hide(&mut self) -> Result<(), String>;
}

fn restore_show_focus<W: WindowOperations>(window: &mut W) -> Result<(), String> {
    let mut errors = Vec::new();
    if let Err(error) = window.unminimize() {
        errors.push(format!("unminimize failed: {error}"));
    }
    if let Err(error) = window.show() {
        errors.push(format!("show failed: {error}"));
    }
    if let Err(error) = window.focus() {
        errors.push(format!("focus failed: {error}"));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn apply_startup_window<W: WindowOperations>(
    window: &mut W,
    database_ready: bool,
    start_minimized: bool,
) {
    match startup_window_mode(database_ready, start_minimized) {
        StartupWindowMode::Normal => {
            let _ = restore_show_focus(window);
        }
        StartupWindowMode::Minimized => {
            if window.show().is_err() || window.minimize().is_err() {
                let _ = restore_show_focus(window);
            }
        }
    }
}

fn should_intercept_close(
    close_to_tray_requested: bool,
    tray_available: bool,
    tray_active: bool,
    explicit_quit: bool,
) -> bool {
    close_to_tray_requested && tray_available && tray_active && !explicit_quit
}

fn try_hide_to_tray<W: WindowOperations>(window: &mut W) -> Result<(), String> {
    window.hide()
}

trait TrayOperations {
    fn create(&mut self) -> Result<(), String>;
    fn remove(&mut self) -> Result<(), String>;
}

fn apply_close_to_tray_transition<T, F>(
    tray: &mut T,
    tray_active: bool,
    requested: bool,
    persist: F,
) -> AppResult<()>
where
    T: TrayOperations,
    F: FnOnce() -> AppResult<()>,
{
    if requested {
        let created = !tray_active;
        if created {
            tray.create()?;
        }
        if let Err(error) = persist() {
            if created {
                let _ = tray.remove();
            }
            return Err(error);
        }
        Ok(())
    } else {
        persist()?;
        if tray_active {
            tray.remove()?;
        }
        Ok(())
    }
}

trait AutostartOperations {
    fn enable(&mut self) -> Result<(), String>;
    fn disable(&mut self) -> Result<(), String>;
    fn is_enabled(&mut self) -> Result<bool, String>;
}

fn query_autostart<A: AutostartOperations>(autostart: &mut A) -> (Option<bool>, Option<String>) {
    match autostart.is_enabled() {
        Ok(enabled) => (Some(enabled), None),
        Err(error) => (
            None,
            Some(format!(
                "Could not query the operating-system login startup registration: {error}"
            )),
        ),
    }
}

fn set_and_verify_autostart<A: AutostartOperations>(
    autostart: &mut A,
    requested: bool,
) -> (Option<bool>, Option<String>) {
    let operation = if requested {
        autostart.enable()
    } else {
        autostart.disable()
    };
    let verification = autostart.is_enabled();
    let observed = verification.as_ref().ok().copied();
    let mut errors = Vec::new();

    if let Err(error) = operation {
        errors.push(format!(
            "Could not {} the operating-system login startup registration: {error}",
            if requested { "enable" } else { "disable" }
        ));
    }
    if let Err(error) = verification {
        errors.push(format!(
            "Could not verify the operating-system login startup registration after the attempt: {error}"
        ));
    } else if observed != Some(requested) {
        errors.push(format!(
            "Login startup verification mismatch: requested {}, but the operating system reports {}.",
            if requested { "enabled" } else { "disabled" },
            if observed == Some(true) { "enabled" } else { "disabled" }
        ));
    }

    (observed, (!errors.is_empty()).then(|| errors.join(" ")))
}

struct TauriWindowOperations<R: Runtime> {
    window: WebviewWindow<R>,
}

impl<R: Runtime> WindowOperations for TauriWindowOperations<R> {
    fn unminimize(&mut self) -> Result<(), String> {
        self.window.unminimize().map_err(|error| error.to_string())
    }

    fn show(&mut self) -> Result<(), String> {
        self.window.show().map_err(|error| error.to_string())
    }

    fn focus(&mut self) -> Result<(), String> {
        self.window.set_focus().map_err(|error| error.to_string())
    }

    fn minimize(&mut self) -> Result<(), String> {
        self.window.minimize().map_err(|error| error.to_string())
    }

    fn hide(&mut self) -> Result<(), String> {
        self.window.hide().map_err(|error| error.to_string())
    }
}

struct TauriTrayOperations<'a, R: Runtime> {
    app: &'a AppHandle<R>,
    state: &'a DesktopState,
}

impl<R: Runtime> TrayOperations for TauriTrayOperations<'_, R> {
    fn create(&mut self) -> Result<(), String> {
        create_tray(self.app).map(|_| self.state.set_tray_available(true)).map_err(|error| {
            let message = format!(
                "System tray initialization failed: {error}. Closing the main window will exit normally."
            );
            self.state.set_tray_error(message.clone());
            message
        })
    }

    fn remove(&mut self) -> Result<(), String> {
        self.app.remove_tray_by_id(TRAY_ID);
        self.state.set_tray_available(false);
        Ok(())
    }
}

struct SystemAutostart<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> AutostartOperations for SystemAutostart<R> {
    fn enable(&mut self) -> Result<(), String> {
        self.app
            .autolaunch()
            .enable()
            .map_err(|error| error.to_string())
    }

    fn disable(&mut self) -> Result<(), String> {
        self.app
            .autolaunch()
            .disable()
            .map_err(|error| error.to_string())
    }

    fn is_enabled(&mut self) -> Result<bool, String> {
        self.app
            .autolaunch()
            .is_enabled()
            .map_err(|error| error.to_string())
    }
}

fn create_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if app.tray_by_id(TRAY_ID).is_some() {
        return Ok(());
    }

    let open = MenuItem::with_id(app, TRAY_OPEN_ID, "Open SSH-Buddy", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let quit = MenuItem::with_id(app, TRAY_QUIT_ID, "Quit SSH-Buddy", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let menu = Menu::with_items(app, &[&open, &quit]).map_err(|error| error.to_string())?;
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("SSH-Buddy")
        .on_menu_event(|app, event| match event.id().as_ref() {
            TRAY_OPEN_ID => restore_main_window(app),
            TRAY_QUIT_ID => {
                app.state::<DesktopState>().set_explicit_quit();
                app.exit(0);
            }
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app).map_err(|error| error.to_string())?;
    Ok(())
}

fn restore_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let mut operations = TauriWindowOperations { window };
        let _ = restore_show_focus(&mut operations);
    }
}

pub fn initialize<R: Runtime>(app: &AppHandle<R>, database_ready: bool, settings: &AppSettings) {
    let state = app.state::<DesktopState>();
    let mut tray = TauriTrayOperations {
        app,
        state: state.inner(),
    };
    match tray.create() {
        Ok(()) if !settings.close_to_tray => {
            let _ = tray.remove();
        }
        Ok(()) => {}
        Err(_) => {}
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let mut operations = TauriWindowOperations { window };
        apply_startup_window(&mut operations, database_ready, settings.start_minimized);
    }
}

pub fn handle_close_request<R: Runtime>(
    app: &AppHandle<R>,
    window: &WebviewWindow<R>,
    api: &tauri::CloseRequestApi,
) {
    let state = app.state::<DesktopState>();
    let runtime = state.snapshot();
    if !should_intercept_close(
        runtime.close_to_tray_requested,
        runtime.tray_available,
        runtime.tray_active,
        runtime.explicit_quit,
    ) {
        return;
    }

    let mut operations = TauriWindowOperations {
        window: window.clone(),
    };
    match try_hide_to_tray(&mut operations) {
        Ok(()) => api.prevent_close(),
        Err(error) => {
            app.remove_tray_by_id(TRAY_ID);
            state.set_tray_error(format!(
                "Could not hide the main window: {error}. Close-to-tray was bypassed so the application can exit normally."
            ));
        }
    }
}

pub fn save_settings<R: Runtime>(
    app: &AppHandle<R>,
    db: &Database,
    input: AppSettings,
) -> AppResult<AppSettings> {
    let state = app.state::<DesktopState>();
    let tray_active = state.snapshot().tray_active;
    let mut tray = TauriTrayOperations {
        app,
        state: state.inner(),
    };
    let requested = input.close_to_tray;
    let mut saved = None;
    apply_close_to_tray_transition(&mut tray, tray_active, requested, || {
        let settings = db.save_settings(input.clone())?;
        saved = Some(settings);
        Ok(())
    })?;
    state.set_close_to_tray_requested(requested);
    saved.ok_or_else(|| "Settings persistence did not return a result".to_string())
}

fn status_with_autostart(
    state: &DesktopState,
    autostart: (Option<bool>, Option<String>),
) -> DesktopBehaviorStatus {
    let runtime = state.snapshot();
    DesktopBehaviorStatus {
        autostart_enabled: autostart.0,
        autostart_error: autostart.1,
        tray_available: runtime.tray_available,
        tray_error: runtime.tray_error,
    }
}

#[tauri::command]
pub fn get_desktop_behavior_status(
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> DesktopBehaviorStatus {
    let mut autostart = SystemAutostart { app };
    status_with_autostart(state.inner(), query_autostart(&mut autostart))
}

#[tauri::command]
pub fn set_autostart_enabled(
    enabled: bool,
    app: AppHandle,
    state: State<'_, DesktopState>,
) -> DesktopBehaviorStatus {
    let mut autostart = SystemAutostart { app };
    status_with_autostart(
        state.inner(),
        set_and_verify_autostart(&mut autostart, enabled),
    )
}

pub fn handle_second_instance<R: Runtime>(app: &AppHandle<R>) {
    restore_main_window(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeWindow {
        calls: Vec<&'static str>,
        fail_show: bool,
        fail_minimize: bool,
        fail_hide: bool,
    }

    impl WindowOperations for FakeWindow {
        fn unminimize(&mut self) -> Result<(), String> {
            self.calls.push("unminimize");
            Ok(())
        }

        fn show(&mut self) -> Result<(), String> {
            self.calls.push("show");
            if self.fail_show {
                Err("show".to_string())
            } else {
                Ok(())
            }
        }

        fn focus(&mut self) -> Result<(), String> {
            self.calls.push("focus");
            Ok(())
        }

        fn minimize(&mut self) -> Result<(), String> {
            self.calls.push("minimize");
            if self.fail_minimize {
                Err("minimize".to_string())
            } else {
                Ok(())
            }
        }

        fn hide(&mut self) -> Result<(), String> {
            self.calls.push("hide");
            if self.fail_hide {
                Err("hide".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[derive(Default)]
    struct FakeTray {
        calls: Vec<&'static str>,
        fail_create: bool,
    }

    impl TrayOperations for FakeTray {
        fn create(&mut self) -> Result<(), String> {
            self.calls.push("create");
            if self.fail_create {
                Err("tray unavailable".to_string())
            } else {
                Ok(())
            }
        }

        fn remove(&mut self) -> Result<(), String> {
            self.calls.push("remove");
            Ok(())
        }
    }

    struct FakeAutostart {
        operation: Result<(), String>,
        observed: Result<bool, String>,
        calls: Vec<&'static str>,
    }

    impl AutostartOperations for FakeAutostart {
        fn enable(&mut self) -> Result<(), String> {
            self.calls.push("enable");
            self.operation.clone()
        }

        fn disable(&mut self) -> Result<(), String> {
            self.calls.push("disable");
            self.operation.clone()
        }

        fn is_enabled(&mut self) -> Result<bool, String> {
            self.calls.push("query");
            self.observed.clone()
        }
    }

    #[test]
    fn startup_decisions_cover_database_and_minimized_combinations() {
        assert_eq!(startup_window_mode(true, false), StartupWindowMode::Normal);
        assert_eq!(
            startup_window_mode(true, true),
            StartupWindowMode::Minimized
        );
        assert_eq!(startup_window_mode(false, false), StartupWindowMode::Normal);
        assert_eq!(startup_window_mode(false, true), StartupWindowMode::Normal);
    }

    #[test]
    fn startup_applies_normal_minimized_and_fallback_window_operations() {
        let mut normal = FakeWindow::default();
        apply_startup_window(&mut normal, true, false);
        assert_eq!(normal.calls, ["unminimize", "show", "focus"]);

        let mut minimized = FakeWindow::default();
        apply_startup_window(&mut minimized, true, true);
        assert_eq!(minimized.calls, ["show", "minimize"]);

        let mut fallback = FakeWindow {
            fail_minimize: true,
            ..Default::default()
        };
        apply_startup_window(&mut fallback, true, true);
        assert_eq!(
            fallback.calls,
            ["show", "minimize", "unminimize", "show", "focus"]
        );

        let mut database_failure = FakeWindow::default();
        apply_startup_window(&mut database_failure, false, true);
        assert_eq!(database_failure.calls, ["unminimize", "show", "focus"]);
    }

    #[test]
    fn close_decisions_cover_preferences_tray_and_explicit_quit() {
        for close_to_tray in [false, true] {
            for tray_available in [false, true] {
                assert_eq!(
                    should_intercept_close(close_to_tray, tray_available, tray_available, false),
                    close_to_tray && tray_available
                );
            }
        }
        assert!(!should_intercept_close(true, true, true, true));
        assert!(!should_intercept_close(true, true, false, false));
    }

    #[test]
    fn hide_failure_falls_back_to_normal_close() {
        let mut window = FakeWindow {
            fail_hide: true,
            ..Default::default()
        };
        assert!(try_hide_to_tray(&mut window).is_err());
        assert_eq!(window.calls, ["hide"]);
    }

    #[test]
    fn open_and_second_instance_restore_hidden_or_minimized_windows() {
        for _initial_state in ["visible", "minimized", "hidden"] {
            let mut window = FakeWindow::default();
            restore_show_focus(&mut window).unwrap();
            assert_eq!(window.calls, ["unminimize", "show", "focus"]);
        }
    }

    #[test]
    fn close_to_tray_enable_rolls_back_new_tray_on_persistence_failure() {
        let mut tray = FakeTray::default();
        let error = apply_close_to_tray_transition(&mut tray, false, true, || {
            Err("database unavailable".to_string())
        })
        .unwrap_err();
        assert_eq!(error, "database unavailable");
        assert_eq!(tray.calls, ["create", "remove"]);
    }

    #[test]
    fn close_to_tray_rejects_enable_when_tray_is_unavailable() {
        let mut tray = FakeTray {
            fail_create: true,
            ..Default::default()
        };
        let mut persisted = false;
        let error = apply_close_to_tray_transition(&mut tray, false, true, || {
            persisted = true;
            Ok(())
        })
        .unwrap_err();
        assert_eq!(error, "tray unavailable");
        assert!(!persisted);
    }

    #[test]
    fn disabling_close_to_tray_persists_then_removes_the_tray() {
        let mut tray = FakeTray::default();
        let persisted = std::cell::Cell::new(false);
        apply_close_to_tray_transition(&mut tray, true, false, || {
            persisted.set(true);
            Ok(())
        })
        .unwrap();
        assert!(persisted.get());
        assert_eq!(tray.calls, ["remove"]);
    }

    #[test]
    fn autostart_enable_disable_success_is_verified() {
        for requested in [true, false] {
            let mut fake = FakeAutostart {
                operation: Ok(()),
                observed: Ok(requested),
                calls: Vec::new(),
            };
            assert_eq!(
                set_and_verify_autostart(&mut fake, requested),
                (Some(requested), None)
            );
            assert_eq!(
                fake.calls,
                [if requested { "enable" } else { "disable" }, "query"]
            );
        }
    }

    #[test]
    fn autostart_operation_failure_still_queries_actual_state() {
        let mut fake = FakeAutostart {
            operation: Err("permission denied".to_string()),
            observed: Ok(false),
            calls: Vec::new(),
        };
        let (observed, error) = set_and_verify_autostart(&mut fake, true);
        assert_eq!(observed, Some(false));
        let error = error.unwrap();
        assert!(error.contains("permission denied"));
        assert!(error.contains("verification mismatch"));
        assert_eq!(fake.calls, ["enable", "query"]);
    }

    #[test]
    fn autostart_query_failure_reports_unknown_post_state() {
        let mut fake = FakeAutostart {
            operation: Ok(()),
            observed: Err("registry unavailable".to_string()),
            calls: Vec::new(),
        };
        let (observed, error) = set_and_verify_autostart(&mut fake, true);
        assert_eq!(observed, None);
        assert!(error.unwrap().contains("Could not verify"));
    }

    #[test]
    fn autostart_mismatched_observed_state_is_an_error() {
        let mut fake = FakeAutostart {
            operation: Ok(()),
            observed: Ok(false),
            calls: Vec::new(),
        };
        let (observed, error) = set_and_verify_autostart(&mut fake, true);
        assert_eq!(observed, Some(false));
        assert!(error.unwrap().contains("verification mismatch"));
    }
}
