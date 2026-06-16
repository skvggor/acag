slint::include_modules!();

use std::rc::Rc;

use anyhow::Result;
use slint::{Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel};

use article_cover_art_generator::cover::config::CoverConfig;
use article_cover_art_generator::cover::layouts::Layout;
use article_cover_art_generator::cover::render_cover_svg;
use article_cover_art_generator::design::patterns::Pattern;
use article_cover_art_generator::design::themes::ThemeName;
use article_cover_art_generator::{export, raster};

/// Preview is rasterized smaller than the 2160² export for snappy live updates.
const PREVIEW_PIXELS: u32 = 768;

fn config_from_ui(ui: &AppWindow) -> CoverConfig {
    let theme = ThemeName::ALL
        .get(ui.get_theme_index().max(0) as usize)
        .copied()
        .unwrap_or(ThemeName::Terracotta);
    let pattern = Pattern::ALL
        .get(ui.get_pattern_index().max(0) as usize)
        .copied()
        .unwrap_or(Pattern::Seigaiha);
    let layout = Layout::ALL
        .get(ui.get_layout_index().max(0) as usize)
        .copied()
        .unwrap_or(Layout::Editorial);
    CoverConfig {
        title: ui.get_title_text().to_string(),
        category: ui.get_category_text().to_string(),
        date: ui.get_date_text().to_string(),
        number: ui.get_number_text().to_string(),
        brand: ui.get_brand_text().to_string(),
        theme,
        pattern,
        layout,
        grain: ui.get_grain_on(),
    }
}

fn refresh_preview(ui: &AppWindow) {
    let svg = render_cover_svg(&config_from_ui(ui));
    match raster::render_to_rgba(&svg, PREVIEW_PIXELS) {
        Ok((width, height, rgba)) => {
            let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
            buffer.make_mut_bytes().copy_from_slice(&rgba);
            ui.set_preview(Image::from_rgba8(buffer));
        }
        Err(error) => ui.set_status(SharedString::from(format!("Preview error: {error}"))),
    }
}

fn string_model(items: Vec<&'static str>) -> ModelRc<SharedString> {
    Rc::new(VecModel::from(
        items
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>(),
    ))
    .into()
}

fn main() -> Result<()> {
    let ui = AppWindow::new()?;

    ui.set_themes(string_model(
        ThemeName::ALL.iter().map(|t| t.label()).collect(),
    ));
    ui.set_patterns(string_model(
        Pattern::ALL.iter().map(|p| p.label()).collect(),
    ));
    ui.set_layouts(string_model(
        Layout::ALL.iter().map(|l| l.label()).collect(),
    ));

    let defaults = CoverConfig::default();
    ui.set_title_text(defaults.title.into());
    ui.set_category_text(defaults.category.into());
    ui.set_date_text(defaults.date.into());
    ui.set_number_text(defaults.number.into());
    ui.set_brand_text(defaults.brand.into());
    ui.set_grain_on(defaults.grain);

    ui.on_changed({
        let handle = ui.as_weak();
        move || {
            if let Some(ui) = handle.upgrade() {
                refresh_preview(&ui);
            }
        }
    });

    ui.on_omakase({
        let handle = ui.as_weak();
        move || {
            if let Some(ui) = handle.upgrade() {
                ui.set_theme_index(fastrand::i32(0..ThemeName::ALL.len() as i32));
                ui.set_pattern_index(fastrand::i32(0..Pattern::ALL.len() as i32));
                ui.set_layout_index(fastrand::i32(0..Layout::ALL.len() as i32));
                ui.set_grain_on(fastrand::bool());
                refresh_preview(&ui);
            }
        }
    });

    ui.on_export_png({
        let handle = ui.as_weak();
        move || {
            if let Some(ui) = handle.upgrade() {
                let status = match export::export_png(&config_from_ui(&ui)) {
                    Ok(path) => format!("Saved PNG → {}", path.display()),
                    Err(error) => format!("Export failed: {error}"),
                };
                ui.set_status(status.into());
            }
        }
    });

    ui.on_export_svg({
        let handle = ui.as_weak();
        move || {
            if let Some(ui) = handle.upgrade() {
                let status = match export::export_svg(&config_from_ui(&ui)) {
                    Ok(path) => format!("Saved SVG → {}", path.display()),
                    Err(error) => format!("Export failed: {error}"),
                };
                ui.set_status(status.into());
            }
        }
    });

    refresh_preview(&ui);
    ui.run()?;
    Ok(())
}
