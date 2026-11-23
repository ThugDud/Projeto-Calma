//! Backend StatusNotifierItem para o KDE Plasma.
//!
//! O `ksni` cuida do protocolo D-Bus (`org.kde.StatusNotifierItem`,
//! `com.canonical.dbusmenu` e propriedades). Este módulo apenas conecta o
//! estado do Calma ao ícone, ao tooltip e ao menu.

use super::{TrayBackend, TrayBackendKind, TrayHandle, TrayIcons, TrayOptions, TraySummary};
use crate::desktop::terminal::TuiLauncher;
use crate::errors::{AppError, AppResult};
use ksni::blocking::{Handle, TrayMethods};
use ksni::{Category, Icon, Status, ToolTip, Tray};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Backend baseado no StatusNotifierItem do KDE/Freedesktop.
#[derive(Debug, Clone, Copy, Default)]
pub struct KdeTrayBackend;

impl TrayBackend for KdeTrayBackend {
    fn kind(&self) -> TrayBackendKind {
        TrayBackendKind::KdeStatusNotifier
    }

    fn is_available(&self) -> bool {
        session_bus_available()
    }

    fn start(&self, options: TrayOptions) -> AppResult<Box<dyn TrayHandle>> {
        let tray = CalmaTray {
            summary: options.summary,
            launcher: options.launcher,
            stop_requested: options.stop_requested,
        };

        // `assume_sni_available` mantém o serviço vivo se o watcher do KDE
        // ainda não estiver pronto no login, registrando-se quando ele voltar.
        let handle = tray.assume_sni_available(true).spawn().map_err(|err| {
            AppError::Tray(format!(
                "não foi possível registrar o ícone no KDE: {}",
                err
            ))
        })?;

        Ok(Box::new(KdeTrayHandle { handle }))
    }
}

/// Controle do ícone registrado no KDE.
struct KdeTrayHandle {
    handle: Handle<CalmaTray>,
}

impl TrayHandle for KdeTrayHandle {
    fn update(&mut self, summary: TraySummary) {
        let _ = self.handle.update(|tray| {
            tray.summary = summary;
        });
    }

    fn is_closed(&self) -> bool {
        self.handle.is_closed()
    }

    fn shutdown(&mut self) {
        self.handle.shutdown().wait();
    }
}

struct CalmaTray {
    summary: TraySummary,
    launcher: Arc<dyn TuiLauncher>,
    stop_requested: Arc<AtomicBool>,
}

impl CalmaTray {
    fn current_icon(&self) -> &'static super::TrayIcon {
        TrayIcons::builtin().for_state(self.summary.state)
    }

    fn open_tui(&self) {
        if let Err(err) = self.launcher.launch() {
            eprintln!("Calma: {}", err);
        }
    }

    fn request_shutdown(&mut self) {
        self.stop_requested.store(true, Ordering::SeqCst);
    }
}

impl Tray for CalmaTray {
    fn id(&self) -> String {
        "calma".to_string()
    }

    fn title(&self) -> String {
        self.summary.tooltip_title()
    }

    fn category(&self) -> Category {
        Category::ApplicationStatus
    }

    fn status(&self) -> Status {
        if self.summary.state.needs_attention() {
            Status::NeedsAttention
        } else {
            Status::Active
        }
    }

    fn icon_name(&self) -> String {
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        vec![self.current_icon().to_ksni()]
    }

    fn attention_icon_pixmap(&self) -> Vec<Icon> {
        vec![TrayIcons::builtin().late.to_ksni()]
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            icon_name: String::new(),
            icon_pixmap: vec![self.current_icon().to_ksni()],
            title: self.summary.tooltip_title(),
            description: self.summary.tooltip_description(),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.open_tui();
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        self.open_tui();
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        vec![
            StandardItem {
                label: self.summary.status_line(),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Abrir Calma".to_string(),
                icon_name: "utilities-terminal".to_string(),
                activate: Box::new(|tray: &mut Self| tray.open_tui()),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Sair do daemon".to_string(),
                icon_name: "application-exit".to_string(),
                activate: Box::new(|tray: &mut Self| tray.request_shutdown()),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn session_bus_available() -> bool {
    if std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some_and(|value| !value.is_empty()) {
        return true;
    }

    std::env::var_os("XDG_RUNTIME_DIR")
        .map(|dir| Path::new(&dir).join("bus").exists())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::tray::TrayState;
    use std::sync::atomic::AtomicUsize;

    #[derive(Debug, Clone)]
    struct CountingLauncher {
        calls: Arc<AtomicUsize>,
    }

    impl TuiLauncher for CountingLauncher {
        fn launch(&self) -> AppResult<()> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn tray(calls: Arc<AtomicUsize>, stop: Arc<AtomicBool>) -> CalmaTray {
        CalmaTray {
            summary: TraySummary::from_counts(1, 0, 0),
            launcher: Arc::new(CountingLauncher { calls }),
            stop_requested: stop,
        }
    }

    #[test]
    fn activate_opens_the_tui() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut tray = tray(calls.clone(), Arc::new(AtomicBool::new(false)));

        tray.activate(0, 0);
        tray.secondary_activate(0, 0);

        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn menu_open_item_opens_the_tui() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mut tray = tray(calls.clone(), Arc::new(AtomicBool::new(false)));

        let menu = tray.menu();
        let open = menu
            .iter()
            .find_map(|item| match item {
                ksni::MenuItem::Standard(item) if item.label == "Abrir Calma" => Some(item),
                _ => None,
            })
            .expect("item Abrir Calma");

        (open.activate)(&mut tray);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn menu_exit_item_requests_daemon_shutdown() {
        let stop = Arc::new(AtomicBool::new(false));
        let mut tray = tray(Arc::new(AtomicUsize::new(0)), stop.clone());

        let menu = tray.menu();
        let exit = menu
            .iter()
            .find_map(|item| match item {
                ksni::MenuItem::Standard(item) if item.label == "Sair do daemon" => Some(item),
                _ => None,
            })
            .expect("item Sair do daemon");

        (exit.activate)(&mut tray);
        assert!(stop.load(Ordering::SeqCst));
    }

    #[test]
    fn late_tasks_use_attention_status_and_late_icon() {
        let tray = CalmaTray {
            summary: TraySummary::from_counts(0, 2, 0),
            launcher: Arc::new(CountingLauncher {
                calls: Arc::new(AtomicUsize::new(0)),
            }),
            stop_requested: Arc::new(AtomicBool::new(false)),
        };

        assert_eq!(tray.summary.state, TrayState::LateVibes);
        assert_eq!(tray.status(), Status::NeedsAttention);

        let pixmap = tray.icon_pixmap();
        assert_eq!(pixmap.len(), 1);
        assert_eq!(pixmap[0].width, 32);
        assert_eq!(pixmap[0].height, 32);
        assert_eq!(pixmap[0].data, TrayIcons::builtin().late.argb);
    }
}
