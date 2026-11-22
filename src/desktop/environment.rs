//! Captura das variáveis de ambiente relevantes para a sessão gráfica.
//!
//! A captura é separada da lógica de detecção para que os testes possam
//! injetar snapshots sintéticos sem alterar o ambiente do processo.

/// Valores de ambiente observados no momento da detecção.
///
/// Todos os campos são opcionais e vazios são normalizados para `None`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvironmentSnapshot {
    pub xdg_current_desktop: Option<String>,
    pub xdg_session_desktop: Option<String>,
    pub desktop_session: Option<String>,
    pub kde_session_version: Option<String>,
    pub xdg_session_type: Option<String>,
    pub dbus_session_bus_address: Option<String>,
    pub wayland_display: Option<String>,
    pub display: Option<String>,
}

impl EnvironmentSnapshot {
    /// Lê o ambiente real do processo atual.
    ///
    /// Variáveis ausentes, vazias ou compostas apenas por espaços são
    /// tratadas como não definidas.
    pub fn from_current() -> Self {
        Self {
            xdg_current_desktop: read("XDG_CURRENT_DESKTOP"),
            xdg_session_desktop: read("XDG_SESSION_DESKTOP"),
            desktop_session: read("DESKTOP_SESSION"),
            kde_session_version: read("KDE_SESSION_VERSION"),
            xdg_session_type: read("XDG_SESSION_TYPE"),
            dbus_session_bus_address: read("DBUS_SESSION_BUS_ADDRESS"),
            wayland_display: read("WAYLAND_DISPLAY"),
            display: read("DISPLAY"),
        }
    }
}

fn read(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
