//! Decodificação e embutimento dos ícones da bandeja.
//!
//! Os PNGs ficam em `src/calma icons` e são embutidos no binário. A
//! decodificação acontece uma única vez, de forma preguiçosa, e o resultado
//! já fica no formato ARGB32 esperado pelo StatusNotifierItem.

use crate::errors::{AppError, AppResult};
use std::io::Cursor;
use std::sync::LazyLock;

/// Imagem ARGB32 em ordem de bytes de rede (`A`, `R`, `G`, `B`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayIcon {
    pub width: i32,
    pub height: i32,
    pub argb: Vec<u8>,
}

impl TrayIcon {
    /// Decodifica um PNG para ARGB32.
    pub fn decode_png(bytes: &[u8]) -> AppResult<Self> {
        let mut decoder = png::Decoder::new(Cursor::new(bytes));
        decoder.set_transformations(
            png::Transformations::normalize_to_color8() | png::Transformations::ALPHA,
        );

        let mut reader = decoder.read_info().map_err(|err| {
            AppError::Tray(format!("não foi possível ler o ícone embutido: {}", err))
        })?;

        let buffer_size = reader.output_buffer_size().ok_or_else(|| {
            AppError::Tray("o ícone embutido possui dimensões inválidas".to_string())
        })?;
        let mut buffer = vec![0; buffer_size];
        let info = reader.next_frame(&mut buffer).map_err(|err| {
            AppError::Tray(format!(
                "não foi possível decodificar o ícone embutido: {}",
                err
            ))
        })?;

        let pixels = &buffer[..info.buffer_size()];
        let argb = match info.color_type {
            png::ColorType::Rgba => rgba_to_argb(pixels),
            png::ColorType::Rgb => rgb_to_argb(pixels),
            png::ColorType::GrayscaleAlpha => grayscale_alpha_to_argb(pixels),
            png::ColorType::Grayscale => grayscale_to_argb(pixels),
            png::ColorType::Indexed => {
                return Err(AppError::Tray(
                    "o ícone embutido não pôde ser expandido para RGBA".to_string(),
                ));
            }
        };

        let expected = info.width as usize * info.height as usize * 4;
        if argb.len() != expected {
            return Err(AppError::Tray(format!(
                "o ícone embutido tem {} bytes, esperado {}",
                argb.len(),
                expected
            )));
        }

        Ok(Self {
            width: info.width as i32,
            height: info.height as i32,
            argb,
        })
    }

    /// Converte para o tipo de ícone aceito pelo backend KDE.
    pub fn to_ksni(&self) -> ksni::Icon {
        ksni::Icon {
            width: self.width,
            height: self.height,
            data: self.argb.clone(),
        }
    }
}

/// Conjunto de ícones embutidos do Calma.
#[derive(Debug)]
pub struct TrayIcons {
    pub good: TrayIcon,
    pub attention: TrayIcon,
    pub late: TrayIcon,
}

impl TrayIcons {
    /// Retorna o conjunto embutido, decodificado apenas uma vez.
    pub fn builtin() -> &'static Self {
        static ICONS: LazyLock<TrayIcons> = LazyLock::new(|| TrayIcons {
            good: decode_builtin(include_bytes!(
                "../../calma icons/calma good vibes favicon.png"
            )),
            attention: decode_builtin(include_bytes!(
                "../../calma icons/calma attention vibes favicon.png"
            )),
            late: decode_builtin(include_bytes!(
                "../../calma icons/calma late vibes favicon.png"
            )),
        });

        &ICONS
    }
}

fn decode_builtin(bytes: &[u8]) -> TrayIcon {
    TrayIcon::decode_png(bytes).expect("ícones embutidos do Calma devem ser PNGs válidos")
}

fn rgba_to_argb(pixels: &[u8]) -> Vec<u8> {
    let mut argb = Vec::with_capacity(pixels.len());
    for pixel in pixels.as_chunks::<4>().0 {
        argb.extend_from_slice(&[pixel[3], pixel[0], pixel[1], pixel[2]]);
    }
    argb
}

fn rgb_to_argb(pixels: &[u8]) -> Vec<u8> {
    let mut argb = Vec::with_capacity(pixels.len() / 3 * 4);
    for pixel in pixels.as_chunks::<3>().0 {
        argb.extend_from_slice(&[0xFF, pixel[0], pixel[1], pixel[2]]);
    }
    argb
}

fn grayscale_alpha_to_argb(pixels: &[u8]) -> Vec<u8> {
    let mut argb = Vec::with_capacity(pixels.len() * 2);
    for pixel in pixels.as_chunks::<2>().0 {
        argb.extend_from_slice(&[pixel[1], pixel[0], pixel[0], pixel[0]]);
    }
    argb
}

fn grayscale_to_argb(pixels: &[u8]) -> Vec<u8> {
    let mut argb = Vec::with_capacity(pixels.len() * 4);
    for pixel in pixels {
        argb.extend_from_slice(&[0xFF, *pixel, *pixel, *pixel]);
    }
    argb
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &[u8] = include_bytes!("../../calma icons/calma good vibes favicon.png");
    const ATTENTION: &[u8] = include_bytes!("../../calma icons/calma attention vibes favicon.png");
    const LATE: &[u8] = include_bytes!("../../calma icons/calma late vibes favicon.png");
    const APP_ICON: &[u8] = include_bytes!("../../calma icons/calma 512x512 icon.png");

    #[test]
    fn builtin_favicons_decode_to_32x32_argb() {
        for (name, bytes) in [("good", GOOD), ("attention", ATTENTION), ("late", LATE)] {
            let icon = TrayIcon::decode_png(bytes).expect("ícone válido");
            assert_eq!(icon.width, 32, "ícone {name}");
            assert_eq!(icon.height, 32, "ícone {name}");
            assert_eq!(icon.argb.len(), 32 * 32 * 4, "ícone {name}");
        }
    }

    #[test]
    fn general_icon_can_be_decoded_for_future_surfaces() {
        let icon = TrayIcon::decode_png(APP_ICON).expect("ícone válido");
        assert_eq!(icon.width, 512);
        assert_eq!(icon.height, 512);
        assert_eq!(icon.argb.len(), 512 * 512 * 4);
    }

    #[test]
    fn builtin_icons_are_cached_in_the_expected_order() {
        let icons = TrayIcons::builtin();
        assert_eq!(icons.good.width, 32);
        assert_eq!(icons.attention.width, 32);
        assert_eq!(icons.late.width, 32);
    }

    #[test]
    fn invalid_png_is_reported_as_a_tray_error() {
        let error = TrayIcon::decode_png(b"not a png").expect_err("deve falhar");
        assert!(matches!(error, AppError::Tray(_)));
    }

    #[test]
    fn conversion_to_ksni_preserves_dimensions_and_data() {
        let icon = TrayIcon::decode_png(GOOD).expect("ícone válido");
        let ksni_icon = icon.to_ksni();
        assert_eq!(ksni_icon.width, 32);
        assert_eq!(ksni_icon.height, 32);
        assert_eq!(ksni_icon.data, icon.argb);
    }
}
