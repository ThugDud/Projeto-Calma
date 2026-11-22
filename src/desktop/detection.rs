//! Detecção do ambiente de desktop Linux.
//!
//! A detecção usa múltiplas fontes e não depende de D-Bus, da TUI ou do
//! daemon. Um ambiente reconhecido não implica que o serviço de notificações
//! esteja disponível: essa verificação pertence ao backend selecionado.

use super::environment::EnvironmentSnapshot;

/// Ambiente de desktop reconhecido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopEnvironment {
    KdePlasma,
    Gnome,
    Xfce,
    Cinnamon,
    Mate,
    Lxqt,
    I3,
    Sway,
    /// Sessão gráfica presente, mas sem identificação reconhecida.
    Unknown(String),
    /// Nenhuma sessão gráfica identificada.
    Headless,
}

impl DesktopEnvironment {
    /// Rótulo legível usado na TUI e em diagnósticos.
    pub fn label(&self) -> String {
        match self {
            DesktopEnvironment::KdePlasma => "KDE Plasma".to_string(),
            DesktopEnvironment::Gnome => "GNOME".to_string(),
            DesktopEnvironment::Xfce => "XFCE".to_string(),
            DesktopEnvironment::Cinnamon => "Cinnamon".to_string(),
            DesktopEnvironment::Mate => "MATE".to_string(),
            DesktopEnvironment::Lxqt => "LXQt".to_string(),
            DesktopEnvironment::I3 => "i3".to_string(),
            DesktopEnvironment::Sway => "Sway".to_string(),
            DesktopEnvironment::Unknown(raw) => format!("Desconhecido ({})", raw),
            DesktopEnvironment::Headless => "Nenhum".to_string(),
        }
    }

    pub fn is_headless(&self) -> bool {
        matches!(self, DesktopEnvironment::Headless)
    }

    pub fn is_kde(&self) -> bool {
        matches!(self, DesktopEnvironment::KdePlasma)
    }
}

/// Tipo de sessão gráfica (servidor de exibição).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Wayland,
    X11,
    Tty,
    Headless,
    Unknown,
}

impl SessionType {
    pub fn label(self) -> &'static str {
        match self {
            SessionType::Wayland => "Wayland",
            SessionType::X11 => "X11",
            SessionType::Tty => "TTY",
            SessionType::Headless => "Headless",
            SessionType::Unknown => "Desconhecida",
        }
    }
}

/// Resultado completo da detecção da sessão.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSession {
    pub environment: DesktopEnvironment,
    pub session_type: SessionType,
    /// Valor bruto da fonte de maior prioridade, útil para diagnóstico.
    pub raw_desktop: Option<String>,
}

impl DesktopSession {
    /// Detecta a sessão real do processo atual.
    pub fn detect() -> Self {
        Self::detect_from(&EnvironmentSnapshot::from_current())
    }

    /// Detecta a sessão a partir de um snapshot injetável.
    ///
    /// A ordem de prioridade é `XDG_CURRENT_DESKTOP`,
    /// `XDG_SESSION_DESKTOP` e `DESKTOP_SESSION`. Se a fonte prioritária
    /// contiver apenas valores desconhecidos, as fontes seguintes ainda são
    /// consultadas. `KDE_SESSION_VERSION` atua como último indício de KDE.
    pub fn detect_from(snapshot: &EnvironmentSnapshot) -> Self {
        let sources = [
            snapshot.xdg_current_desktop.as_deref(),
            snapshot.xdg_session_desktop.as_deref(),
            snapshot.desktop_session.as_deref(),
        ];

        let raw_desktop = sources
            .iter()
            .flatten()
            .map(|value| value.trim().to_string())
            .find(|value| !value.is_empty());

        let mut environment = None;
        for source in sources.into_iter().flatten() {
            for token in split_tokens(source) {
                if let Some(desktop) = classify_token(token) {
                    environment = Some(desktop);
                    break;
                }
            }
            if environment.is_some() {
                break;
            }
        }

        let environment = match environment {
            Some(desktop) => desktop,
            None => {
                if let Some(raw) = raw_desktop.clone() {
                    DesktopEnvironment::Unknown(raw)
                } else if snapshot.kde_session_version.is_some() {
                    DesktopEnvironment::KdePlasma
                } else if has_graphical_indicator(snapshot) {
                    DesktopEnvironment::Unknown("sessão gráfica sem identificação".to_string())
                } else {
                    DesktopEnvironment::Headless
                }
            }
        };

        let session_type = detect_session_type(snapshot, &environment);

        Self {
            environment,
            session_type,
            raw_desktop,
        }
    }
}

fn split_tokens(value: &str) -> impl Iterator<Item = &str> {
    value
        .split([':', ';', ','])
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

fn classify_token(token: &str) -> Option<DesktopEnvironment> {
    let upper = token.to_ascii_uppercase();
    if upper.is_empty() {
        return None;
    }

    if upper.contains("KDE") || upper.contains("PLASMA") {
        Some(DesktopEnvironment::KdePlasma)
    } else if upper.starts_with("GNOME") {
        Some(DesktopEnvironment::Gnome)
    } else if upper.starts_with("XFCE") {
        Some(DesktopEnvironment::Xfce)
    } else if upper.contains("CINNAMON") {
        Some(DesktopEnvironment::Cinnamon)
    } else if upper == "MATE" || upper.starts_with("MATE-") {
        Some(DesktopEnvironment::Mate)
    } else if upper.contains("LXQT") {
        Some(DesktopEnvironment::Lxqt)
    } else if upper == "I3" || upper.starts_with("I3-") || upper == "I3WM" {
        Some(DesktopEnvironment::I3)
    } else if upper.starts_with("SWAY") {
        Some(DesktopEnvironment::Sway)
    } else {
        None
    }
}

fn detect_session_type(
    snapshot: &EnvironmentSnapshot,
    environment: &DesktopEnvironment,
) -> SessionType {
    if let Some(raw) = &snapshot.xdg_session_type {
        match raw.to_ascii_lowercase().as_str() {
            "wayland" => return SessionType::Wayland,
            "x11" => return SessionType::X11,
            "tty" => return SessionType::Tty,
            _ => {}
        }
    }

    if snapshot.wayland_display.is_some() {
        SessionType::Wayland
    } else if snapshot.display.is_some() {
        SessionType::X11
    } else if environment.is_headless() {
        SessionType::Headless
    } else {
        SessionType::Unknown
    }
}

fn has_graphical_indicator(snapshot: &EnvironmentSnapshot) -> bool {
    snapshot.wayland_display.is_some()
        || snapshot.display.is_some()
        || snapshot
            .xdg_session_type
            .as_deref()
            .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "wayland" | "x11"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> EnvironmentSnapshot {
        EnvironmentSnapshot::default()
    }

    #[test]
    fn detects_kde_plasma() {
        let env = EnvironmentSnapshot {
            xdg_current_desktop: Some("KDE".into()),
            xdg_session_type: Some("wayland".into()),
            ..snapshot()
        };

        let session = DesktopSession::detect_from(&env);
        assert_eq!(session.environment, DesktopEnvironment::KdePlasma);
        assert_eq!(session.session_type, SessionType::Wayland);
    }

    #[test]
    fn detects_kde_through_variations() {
        for value in ["KDE:Plasma", "PLASMA", "plasma", "KDE5"] {
            let env = EnvironmentSnapshot {
                xdg_current_desktop: Some(value.into()),
                ..snapshot()
            };
            assert_eq!(
                DesktopSession::detect_from(&env).environment,
                DesktopEnvironment::KdePlasma,
                "valor: {value}"
            );
        }
    }

    #[test]
    fn detects_kde_through_alternative_variables() {
        let env = EnvironmentSnapshot {
            desktop_session: Some("plasma".into()),
            ..snapshot()
        };
        assert_eq!(
            DesktopSession::detect_from(&env).environment,
            DesktopEnvironment::KdePlasma
        );

        let env = EnvironmentSnapshot {
            kde_session_version: Some("6".into()),
            ..snapshot()
        };
        assert_eq!(
            DesktopSession::detect_from(&env).environment,
            DesktopEnvironment::KdePlasma
        );
    }

    #[test]
    fn detects_gnome() {
        let env = EnvironmentSnapshot {
            xdg_current_desktop: Some("ubuntu:GNOME".into()),
            xdg_session_type: Some("x11".into()),
            ..snapshot()
        };
        let session = DesktopSession::detect_from(&env);
        assert_eq!(session.environment, DesktopEnvironment::Gnome);
        assert_eq!(session.session_type, SessionType::X11);
    }

    #[test]
    fn detects_xfce() {
        let env = EnvironmentSnapshot {
            xdg_current_desktop: Some("XFCE".into()),
            ..snapshot()
        };
        assert_eq!(
            DesktopSession::detect_from(&env).environment,
            DesktopEnvironment::Xfce
        );
    }

    #[test]
    fn detects_cinnamon_mate_and_lxqt() {
        let cases = [
            ("X-Cinnamon", DesktopEnvironment::Cinnamon),
            ("MATE", DesktopEnvironment::Mate),
            ("LXQt", DesktopEnvironment::Lxqt),
        ];

        for (value, expected) in cases {
            let env = EnvironmentSnapshot {
                xdg_current_desktop: Some(value.into()),
                ..snapshot()
            };
            assert_eq!(
                DesktopSession::detect_from(&env).environment,
                expected,
                "valor: {value}"
            );
        }
    }

    #[test]
    fn detects_i3_and_sway() {
        let i3 = DesktopSession::detect_from(&EnvironmentSnapshot {
            desktop_session: Some("i3".into()),
            ..snapshot()
        });
        assert_eq!(i3.environment, DesktopEnvironment::I3);

        let sway = DesktopSession::detect_from(&EnvironmentSnapshot {
            xdg_current_desktop: Some("sway".into()),
            wayland_display: Some("wayland-1".into()),
            ..snapshot()
        });
        assert_eq!(sway.environment, DesktopEnvironment::Sway);
        assert_eq!(sway.session_type, SessionType::Wayland);
    }

    #[test]
    fn detects_unknown_environment() {
        let env = EnvironmentSnapshot {
            xdg_current_desktop: Some("meu-desktop-exotico".into()),
            display: Some(":0".into()),
            ..snapshot()
        };
        let session = DesktopSession::detect_from(&env);
        assert_eq!(
            session.environment,
            DesktopEnvironment::Unknown("meu-desktop-exotico".into())
        );
        assert_eq!(session.session_type, SessionType::X11);
    }

    #[test]
    fn missing_environment_variables_is_headless() {
        let session = DesktopSession::detect_from(&snapshot());
        assert_eq!(session.environment, DesktopEnvironment::Headless);
        assert_eq!(session.session_type, SessionType::Headless);
    }

    #[test]
    fn headless_ignores_dbus_alone() {
        let env = EnvironmentSnapshot {
            dbus_session_bus_address: Some("unix:path=/run/user/1000/bus".into()),
            ..snapshot()
        };
        let session = DesktopSession::detect_from(&env);
        assert_eq!(session.environment, DesktopEnvironment::Headless);
    }

    #[test]
    fn conflicting_variables_prefer_xdg_current_desktop() {
        let env = EnvironmentSnapshot {
            xdg_current_desktop: Some("KDE".into()),
            xdg_session_desktop: Some("GNOME".into()),
            desktop_session: Some("xfce".into()),
            ..snapshot()
        };
        assert_eq!(
            DesktopSession::detect_from(&env).environment,
            DesktopEnvironment::KdePlasma
        );
    }

    #[test]
    fn unknown_priority_source_falls_back_to_later_source() {
        let env = EnvironmentSnapshot {
            xdg_current_desktop: Some("desconhecido".into()),
            xdg_session_desktop: Some("GNOME".into()),
            ..snapshot()
        };
        assert_eq!(
            DesktopSession::detect_from(&env).environment,
            DesktopEnvironment::Gnome
        );
    }

    #[test]
    fn graphical_session_without_identification_is_unknown_not_headless() {
        let env = EnvironmentSnapshot {
            wayland_display: Some("wayland-0".into()),
            ..snapshot()
        };
        let session = DesktopSession::detect_from(&env);
        assert!(matches!(
            session.environment,
            DesktopEnvironment::Unknown(_)
        ));
        assert_eq!(session.session_type, SessionType::Wayland);
    }

    #[test]
    fn session_type_falls_back_to_display_variables() {
        let wayland = DesktopSession::detect_from(&EnvironmentSnapshot {
            xdg_current_desktop: Some("KDE".into()),
            wayland_display: Some("wayland-0".into()),
            ..snapshot()
        });
        assert_eq!(wayland.session_type, SessionType::Wayland);

        let x11 = DesktopSession::detect_from(&EnvironmentSnapshot {
            xdg_current_desktop: Some("KDE".into()),
            display: Some(":0".into()),
            ..snapshot()
        });
        assert_eq!(x11.session_type, SessionType::X11);
    }

    #[test]
    fn detection_never_panics_with_inconsistent_values() {
        let env = EnvironmentSnapshot {
            xdg_current_desktop: Some("::;;,,".into()),
            xdg_session_type: Some("algo-estranho".into()),
            ..snapshot()
        };
        let session = DesktopSession::detect_from(&env);
        assert_eq!(session.session_type, SessionType::Unknown);
    }
}
