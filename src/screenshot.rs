use anyhow::Result;
use std::io::Cursor;
use xcap::Window;

pub fn capture_active_window() -> Result<(Vec<u8>, String)> {
    let windows = Window::all()?;
    let active = windows
        .iter()
        .find(|w| {
            !w.is_minimized().unwrap_or(false)
                && w.title().as_deref().unwrap_or("").contains("")
        })
        .and_then(|_| {
            windows.iter().find(|w| {
                let title = w.title().unwrap_or_default();
                !title.is_empty() && !title.contains("Thoth") && !w.is_minimized().unwrap_or(true)
            })
        })
        .or_else(|| {
            windows.iter().find(|w| {
                let title = w.title().unwrap_or_default();
                !title.is_empty() && !w.is_minimized().unwrap_or(true)
            })
        });

    let target = match active {
        Some(w) => w,
        None => {
            anyhow::bail!("no suitable window found for capture");
        }
    };

    let window_title = target.title().unwrap_or_default();
    let image = target.capture_image()?;

    let mut png_bytes = Vec::new();
    {
        let mut cursor = Cursor::new(&mut png_bytes);
        image.write_to(&mut cursor, xcap::image::ImageFormat::Png)?;
    }

    tracing::info!(
        "captured window '{}' ({}x{}), PNG size: {} bytes",
        window_title,
        image.width(),
        image.height(),
        png_bytes.len()
    );

    Ok((png_bytes, window_title))
}
