use std::path::{Path, PathBuf};

/// Gestor de rutas de temas y estilos para Malus DE
pub struct ThemeManager {
    theme_name: String,
}

impl ThemeManager {
    pub fn new(theme_name: impl Into<String>) -> Self {
        Self {
            theme_name: theme_name.into(),
        }
    }

    /// Ruta al directorio base del tema en ~/.config/malus-de/themes/<theme_name>/
    pub fn theme_dir(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Path::new(&home)
            .join(".config")
            .join("malus-de")
            .join("themes")
            .join(&self.theme_name)
    }

    pub fn tokens_css_path(&self) -> PathBuf {
        self.theme_dir().join("tokens.css")
    }

    pub fn dock_css_path(&self) -> PathBuf {
        self.theme_dir().join("dock.css")
    }

    pub fn bar_css_path(&self) -> PathBuf {
        self.theme_dir().join("bar.css")
    }
}
