//! The Home screen: the card PhotoCraft shows instead of a canvas when no document is open.
//!
//! The wordmark and tagline, the New… / Open… buttons, the file-drop hint (on Wayland, where
//! winit 0.30 has no file drops, it points at File › Open and paste instead), the Templates
//! gallery (every category and preset of the New Document dialog, reachable without opening
//! the dialog first), the Recent files list and the Discord and link row — on a card 460 wide
//! that grows with the number of recent files. The screen draws no canvas, so it lives here
//! rather than in [`canvas`].

use egui::{Rect, Sense};

use crate::PhotocraftApp;
use crate::canvas::paint_dots;

pub fn start_screen(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let area = ui.available_rect_before_wrap();
    paint_dots(ui, area, app.session.prefs().interface.show_canvas_dots);
    // File › Open Recent, newest first (on the web there are no paths to reopen).
    let recent: Vec<String> = if cfg!(target_arch = "wasm32") { Vec::new() } else { app.ui.recent_files.iter().take(HOME_RECENT).cloned().collect() };
    let recent_h = if recent.is_empty() { 0.0 } else { 34.0 + recent.len() as f32 * HOME_RECENT_ROW };
    let card = Rect::from_center_size(area.center(), egui::vec2(460.0, 330.0 + recent_h));
    let new_label = crate::shortcuts::command_label(app, "New document…", "file.new");
    let open_label = crate::shortcuts::command_label(app, "Open…", "file.open");
    ui.scope_builder(egui::UiBuilder::new().max_rect(card), |ui| {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                let title = ui.painter().layout_no_wrap(tl!("PhotoCraft").into(), crate::theme::semibold(38.0), t.text);
                let by = ui.painter().layout_no_wrap(tl!("open source").into(), egui::FontId::proportional(13.0), t.text_faint);
                let total = title.size().x + by.size().x + 10.0;
                ui.add_space(((card.width() - total) / 2.0).max(0.0));
                let (r, _) = ui.allocate_exact_size(title.size(), Sense::hover());
                ui.painter().galley(r.min, title, t.text);
                let (r2, _) = ui.allocate_exact_size(egui::vec2(by.size().x + 10.0, r.height()), Sense::hover());
                ui.painter().galley(egui::pos2(r2.left() + 10.0, r.bottom() - by.size().y - 8.0), by, t.text_faint);
            });
            ui.add_space(6.0);
            ui.label(egui::RichText::new(tl!("Create a new document or open an existing file.")).color(t.text_dim).size(14.0));
            ui.add_space(22.0);
            ui.horizontal(|ui| {
                ui.add_space(((card.width() - 2.0 * 190.0 - 12.0) / 2.0).max(0.0));
                ui.spacing_mut().item_spacing.x = 12.0;
                if crate::widgets::primary_button(ui, &new_label, 190.0).clicked() {
                    let fields = app.new_document_fields();
                    app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
                }
                if crate::widgets::secondary_button(ui, &open_label, 190.0).clicked() {
                    let _ = app.open_dialog_file();
                }
            });
            ui.add_space(22.0);
            ui.horizontal(|ui| {
                let msg = start_screen_drop_hint(app);
                let g = ui.painter().layout_no_wrap(msg.to_string(), egui::FontId::proportional(12.5), t.text_faint);
                ui.add_space(((card.width() - g.size().x - 24.0) / 2.0).max(0.0));
                let (r, _) = ui.allocate_exact_size(egui::vec2(18.0, 18.0), Sense::hover());
                crate::icons::paint(ui, r, "image", 15.0, t.text_faint);
                ui.label(egui::RichText::new(msg).color(t.text_faint));
            });
            if !recent.is_empty() {
                ui.add_space(22.0);
                home_recent(app, ui, &recent);
            }
            ui.add_space(26.0);
            crate::links::discord_button(app, ui, 190.0);
            ui.add_space(10.0);
            crate::links::link_row(app, ui);
        });
    });
}

/// Wayland has no native file drops (winit 0.30, #386), so the hint offers File › Open and paste.
pub(crate) fn start_screen_drop_hint(app: &PhotocraftApp) -> std::borrow::Cow<'static, str> {
    if app.services.is_wayland {
        crate::i18n::fmt(tl!("Use File › Open, or paste a copied image with {paste}."), &[("paste", &crate::notices::paste_hint(app))]).into()
    } else {
        tl!("Drop an image or PSD anywhere to open it.").into()
    }
}

/// Recent files listed on the Home screen.
pub const HOME_RECENT: usize = 6;
/// Height of one Home-screen recent-file row.
pub const HOME_RECENT_ROW: f32 = 30.0;

/// The Home screen's "Recent" list: file name, its folder on the right, click to open. The
/// hovered row shows a × that takes it off the list, and right-click offers Remove from Recent
/// and Clear Recent Files (#2691).
pub(crate) fn home_recent(app: &mut PhotocraftApp, ui: &mut egui::Ui, recent: &[String]) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let width = 380.0;
    ui.horizontal(|ui| {
        // Line the heading up with the file icons.
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0) + 8.0);
        ui.label(egui::RichText::new(tl!("Recent")).font(crate::theme::semibold(12.5)).color(t.text_dim));
    });
    ui.add_space(4.0);
    let mut open = None;
    let mut remove = None;
    let mut clear = false;
    for path in recent {
        let name = crate::file_open::display_name(path);
        let folder = std::path::Path::new(path).parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
        let (row, resp) = ui.allocate_exact_size(egui::vec2(width, HOME_RECENT_ROW), Sense::click());
        // The × sits on top of the row, so the row counts as hovered while the pointer is on it.
        let hovered = ui.rect_contains_pointer(row);
        if hovered {
            ui.painter().rect_filled(row, t.radius_sm, t.hover);
        }
        let close = Rect::from_center_size(egui::pos2(row.right() - 16.0, row.center().y), egui::vec2(22.0, 22.0));
        let right = if hovered { close.left() - 4.0 } else { row.right() - 10.0 };
        let icon = Rect::from_center_size(egui::pos2(row.left() + 16.0, row.center().y), egui::vec2(16.0, 16.0));
        crate::icons::paint(ui, icon, "file", 14.0, t.text_dim);
        let x = row.left() + 32.0;
        let name_g = crate::tab_strip::elided(ui, &name, egui::FontId::proportional(12.5), t.text, 170.0);
        let name_w = name_g.size().x;
        ui.painter().galley(egui::pos2(x, row.center().y - name_g.size().y / 2.0), name_g, t.text);
        let folder_max = (right - (x + name_w + 14.0)).max(0.0);
        if folder_max > 20.0 && !folder.is_empty() {
            let fg = crate::tab_strip::elided(ui, &folder, egui::FontId::proportional(11.5), t.text_faint, folder_max);
            ui.painter().galley(egui::pos2(right - fg.size().x, row.center().y - fg.size().y / 2.0), fg, t.text_faint);
        }
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &name));
        let resp = resp.on_hover_text(path).on_hover_cursor(egui::CursorIcon::PointingHand);
        if hovered {
            let x = ui.interact(close, resp.id.with("remove"), Sense::click());
            let label = crate::i18n::fmt(tl!("Remove {name} from Recent"), &[("name", &name)]);
            x.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
            let color = crate::icons::button_chrome(ui, close, false, x.hovered());
            crate::icons::paint(ui, close, "x", 12.0, color);
            if x.on_hover_text(tl!("Remove from Recent")).clicked() {
                remove = Some(path.clone());
            }
        }
        if resp.clicked() && remove.is_none() {
            open = Some(path.clone());
        }
        resp.context_menu(|ui| {
            if ui.button(tl!("Remove from Recent")).clicked() {
                remove = Some(path.clone());
                ui.close();
            }
            if ui.button(tl!("Clear Recent Files")).clicked() {
                clear = true;
                ui.close();
            }
        });
    }
    if clear {
        app.clear_recent();
    } else if let Some(path) = remove {
        app.remove_recent(&path);
    }
    if let Some(path) = open
        && let Err(e) = app.open_path(&path)
    {
        app.open_failed(&crate::file_open::display_name(&path), &e);
    }
}

/// A Templates card's size: the New Document dialog's preset card, so the same preset looks like
/// the same thing in both places.
pub const TEMPLATE_CARD: egui::Vec2 = egui::vec2(164.0, 112.0);
/// Templates cards per row.
pub const TEMPLATE_COLUMNS: usize = 3;

/// The category `index` names, or Recent (0) when the index is out of range.
///
/// The index is serialised state — a state file can hold any number — so it is resolved before
/// use and never indexed with. Resolving it first also means the fallback is visible in the UI:
/// the Recent tab stays underlined instead of every tab going flat.
fn resolved_category(category: usize) -> usize {
    if category < crate::new_doc_ui::CATEGORIES.len() { category } else { 0 }
}

/// The presets of `category`, or an empty slice when the index is out of range.
pub fn category_presets(category: usize) -> &'static [crate::new_doc_ui::Preset] {
    crate::new_doc_ui::CATEGORIES.get(category).map_or(&[], |c| c.1)
}

/// The Home screen's Templates panel: the New Document dialog's categories as a tab row and their
/// presets as cards; clicking a card opens File › New with that preset applied. The categories, the
/// cards and the Recent rule are the dialog's own — [`crate::new_doc_ui::recent_cards`] for Recent,
/// and the dialog's card drawing for each card — so the two screens cannot drift apart.
///
/// An empty category (or an empty Recent) draws the tab row and nothing else — no error, no
/// placeholder.
pub fn templates(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let category = resolved_category(app.ui.chrome.home_category);
    // The category tabs, drawn exactly as the dialog draws them.
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 18.0;
        for (i, c) in crate::new_doc_ui::CATEGORIES.iter().enumerate() {
            let on = category == i;
            let label = egui::Label::new(egui::RichText::new(tl!(c.0)).size(13.0).color(if on { t.text } else { t.text_dim })).sense(Sense::click());
            let r = ui.add(label);
            if on {
                let under = [r.rect.left_bottom() + egui::vec2(0.0, 3.0), r.rect.right_bottom() + egui::vec2(0.0, 3.0)];
                ui.painter().line_segment(under, egui::Stroke::new(2.0, t.text));
            }
            if r.clicked() {
                app.ui.chrome.home_category = i;
            }
        }
    });
    ui.add_space(6.0);
    let units = app.session.prefs().units_and_rulers.clone();
    let presets: Vec<crate::new_doc_ui::Card> = if category == 0 {
        // Recent is the dialog's rule: the documents created with the dialog, then the built-in
        // defaults they don't cover. An empty field map: no clipboard image was copied to make a
        // card out of here, and a card's click fills the dialog's fields when there are any.
        crate::new_doc_ui::recent_cards(&serde_json::Map::new(), &app.session.presets.recent_documents, &units)
    } else {
        category_presets(category).iter().copied().map(crate::new_doc_ui::Card::Builtin).collect()
    };
    for row in presets.chunks(TEMPLATE_COLUMNS) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for card in row {
                template_card(app, ui, card, &t, &units);
            }
        });
        ui.add_space(8.0);
    }
}

/// One Templates card: drawn by the New Document dialog's own card drawing — the page at the
/// preset's aspect ratio, the preset's name (wrapped and elided so a long translation stays
/// inside the card) and its size in the unit the preset is specified in. Clicking applies it to a
/// fresh set of dialog fields and opens File › New, as the dialog's own card click does.
fn template_card(
    app: &mut PhotocraftApp,
    ui: &mut egui::Ui,
    card: &crate::new_doc_ui::Card,
    t: &crate::theme::Tokens,
    units: &photocraft_engine::prefs::UnitsAndRulers,
) {
    let (w, h, _) = card.size(units);
    let name = tl!(card.title()).to_owned();
    let size = card.size_line(units);
    let (r, resp) = ui.allocate_exact_size(TEMPLATE_CARD, Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &name));
    ui.painter().rect_filled(r, t.radius, if resp.hovered() { t.hover } else { t.field });
    crate::new_doc_ui::page_icon(ui, Rect::from_center_size(egui::pos2(r.center().x, r.top() + 34.0), egui::vec2(40.0, 40.0)), w, h, t);
    let title = crate::new_doc_ui::card_title(ui.painter(), &name, TEMPLATE_CARD.x - 12.0, t.text);
    let elided = title.elided;
    ui.painter().galley(egui::pos2(r.center().x, r.top() + 70.0 - title.size().y / 2.0), title, t.text);
    ui.painter().text(egui::pos2(r.center().x, r.top() + 97.0), egui::Align2::CENTER_CENTER, &size, egui::FontId::proportional(10.5), t.text_faint);
    // A title too long for the card shows in full on hover, as it does in the dialog.
    let resp = if elided { resp.on_hover_text(name) } else { resp };
    if resp.clicked() {
        let mut fields = app.new_document_fields();
        match card {
            crate::new_doc_ui::Card::Builtin(p) => crate::new_doc_ui::apply_preset(&mut fields, p, units),
            crate::new_doc_ui::Card::Recent(s) => crate::new_doc_ui::apply_settings(&mut fields, s),
        }
        app.ui.open_dialog(crate::state::DialogKind::NewDocument, fields);
    }
}
