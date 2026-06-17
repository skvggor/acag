// Don't allocate a console window on Windows for the released GUI binary; keep
// it in debug builds so logs/panics stay visible during development.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use slint::{
    Image, Model, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, Timer, TimerMode, VecModel,
};

use article_cover_art_generator::cover::config::CoverConfig;
use article_cover_art_generator::cover::format::Format;
use article_cover_art_generator::cover::layouts::Layout;
use article_cover_art_generator::cover::render_cover_svg;
use article_cover_art_generator::design::patterns::Pattern;
use article_cover_art_generator::design::themes::ThemeName;
use article_cover_art_generator::raster::{EXPORT_2K, EXPORT_4K};
use article_cover_art_generator::{export, preset, raster};

/// Preview is rasterized smaller than the export for snappy live updates.
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
    let format = Format::ALL
        .get(ui.get_format_index().max(0) as usize)
        .copied()
        .unwrap_or(Format::Square);
    CoverConfig {
        title: ui.get_title_text().to_string(),
        category: ui.get_category_text().to_string(),
        date: ui.get_date_text().to_string(),
        number: ui.get_number_text().to_string(),
        brand: ui.get_brand_text().to_string(),
        theme,
        pattern,
        layout,
        format,
        grain: f64::from(ui.get_grain_value()),
        pattern_strength: f64::from(ui.get_pattern_value()),
    }
}

fn apply_config(ui: &AppWindow, config: &CoverConfig) {
    ui.set_title_text(config.title.clone().into());
    ui.set_category_text(config.category.clone().into());
    ui.set_date_text(config.date.clone().into());
    ui.set_number_text(config.number.clone().into());
    ui.set_brand_text(config.brand.clone().into());
    ui.set_theme_index(config.theme.index() as i32);
    ui.set_pattern_index(config.pattern.index() as i32);
    ui.set_layout_index(config.layout.index() as i32);
    ui.set_format_index(config.format.index() as i32);
    ui.set_grain_value(config.grain as f32);
    ui.set_pattern_value(config.pattern_strength as f32);
}

fn export_size(ui: &AppWindow) -> u32 {
    if ui.get_size_index() == 0 {
        EXPORT_2K
    } else {
        EXPORT_4K
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

fn string_model(items: Vec<&str>) -> ModelRc<SharedString> {
    Rc::new(VecModel::from(
        items
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>(),
    ))
    .into()
}

fn reload_presets(model: &VecModel<SharedString>) {
    let names: Vec<SharedString> = preset::list().into_iter().map(SharedString::from).collect();
    model.set_vec(names);
}

/// Default to Slint's CPU (software) renderer so the app runs everywhere,
/// including machines without a usable OpenGL driver (headless VMs, RDP).
/// For this form-plus-preview UI the cost is imperceptible. Users with working
/// GPU drivers can opt into hardware rendering with `SLINT_BACKEND=winit-femtovg`.
fn default_to_software_renderer() {
    if std::env::var_os("SLINT_BACKEND").is_none() {
        // Safe: set before any backend/window initialization, still single-threaded.
        unsafe { std::env::set_var("SLINT_BACKEND", "winit-software") };
    }
}

fn main() -> Result<()> {
    default_to_software_renderer();

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
    ui.set_sizes(string_model(vec!["2160 · 2K", "4096 · 4K"]));
    ui.set_formats(string_model(
        Format::ALL.iter().map(|f| f.label()).collect(),
    ));

    let presets = Rc::new(VecModel::<SharedString>::default());
    ui.set_presets(presets.clone().into());
    reload_presets(&presets);

    apply_config(&ui, &CoverConfig::default());
    ui.set_size_index(1); // 4K by default
    ui.set_open_after(true);

    // Debounce: coalesce rapid edits (slider drags, typing) into one render
    // ~40 ms after the user stops, instead of rasterizing on every event.
    let preview_timer = Rc::new(Timer::default());
    ui.on_changed({
        let handle = ui.as_weak();
        let timer = preview_timer.clone();
        move || {
            let handle = handle.clone();
            timer.start(
                TimerMode::SingleShot,
                Duration::from_millis(40),
                move || {
                    if let Some(ui) = handle.upgrade() {
                        refresh_preview(&ui);
                    }
                },
            );
        }
    });

    ui.on_omakase({
        let handle = ui.as_weak();
        move || {
            if let Some(ui) = handle.upgrade() {
                let mut config = config_from_ui(&ui);
                config.randomize_style();
                apply_config(&ui, &config);
                refresh_preview(&ui);
            }
        }
    });

    ui.on_export_png({
        let handle = ui.as_weak();
        move || {
            let Some(ui) = handle.upgrade() else { return };
            let config = config_from_ui(&ui);
            let pixels = export_size(&ui);
            let open = ui.get_open_after();
            // Show the loader, then rasterize off the UI thread so it stays responsive.
            ui.set_exporting(true);
            ui.set_status(SharedString::new());
            let weak = handle.clone();
            std::thread::spawn(move || {
                let (message, path) = match export::export_png(&config, pixels) {
                    Ok(path) => (format!("Saved PNG → {}", path.display()), Some(path)),
                    Err(error) => (format!("Export failed: {error}"), None),
                };
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak.upgrade() {
                        ui.set_exporting(false);
                        ui.set_status(message.into());
                        if open && let Some(path) = path {
                            let _ = export::open_in_viewer(&path);
                        }
                    }
                })
                .ok();
            });
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

    ui.on_save_preset({
        let handle = ui.as_weak();
        let presets = presets.clone();
        move || {
            if let Some(ui) = handle.upgrade() {
                let typed = ui.get_preset_name().to_string();
                let name = if typed.trim().is_empty() {
                    ui.get_title_text().to_string()
                } else {
                    typed
                };
                let status = match preset::save(&name, &config_from_ui(&ui)) {
                    Ok(stored) => {
                        reload_presets(&presets);
                        if let Some(index) = preset::list().iter().position(|n| n == &stored) {
                            ui.set_preset_index(index as i32);
                        }
                        format!("Saved preset \"{stored}\"")
                    }
                    Err(error) => format!("Save preset failed: {error}"),
                };
                ui.set_status(status.into());
            }
        }
    });

    ui.on_load_preset({
        let handle = ui.as_weak();
        let presets = presets.clone();
        move || {
            if let Some(ui) = handle.upgrade() {
                let index = ui.get_preset_index().max(0) as usize;
                let Some(name) = presets.row_data(index) else {
                    ui.set_status("Select a saved preset first".into());
                    return;
                };
                match preset::load(name.as_str()) {
                    Ok(config) => {
                        apply_config(&ui, &config);
                        refresh_preview(&ui);
                        ui.set_status(format!("Loaded preset \"{name}\"").into());
                    }
                    Err(error) => ui.set_status(format!("Load preset failed: {error}").into()),
                }
            }
        }
    });

    ui.on_delete_preset({
        let handle = ui.as_weak();
        let presets = presets.clone();
        move || {
            if let Some(ui) = handle.upgrade() {
                let index = ui.get_preset_index().max(0) as usize;
                let Some(name) = presets.row_data(index) else {
                    return;
                };
                let status = match preset::delete(name.as_str()) {
                    Ok(()) => {
                        reload_presets(&presets);
                        ui.set_preset_index(0);
                        format!("Deleted preset \"{name}\"")
                    }
                    Err(error) => format!("Delete preset failed: {error}"),
                };
                ui.set_status(status.into());
            }
        }
    });

    refresh_preview(&ui);
    ui.run()?;
    Ok(())
}
