//! Home screen: the Templates panel and the category state behind it.

use egui::vec2;
use egui_kittest::{Harness, kittest::Queryable};
use photocraft_engine::document_preset_cmds::DocumentSettings;

use crate::PhotocraftApp;
use crate::chrome_ui::ChromeState;
use crate::home_screen;
use crate::new_doc_ui::CATEGORIES;

fn app() -> PhotocraftApp {
    PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default())
}

/// A recently created document, at a size no built-in preset lists, so the dialog names its card
/// "Custom" rather than after a preset.
fn recent(width: u32, height: u32, resolution: f32) -> DocumentSettings {
    DocumentSettings {
        width,
        height,
        resolution,
        mode: "rgb".into(),
        depth: 8,
        background: "white".into(),
        background_color: None,
        unit: "px".into(),
        resolution_unit: "in".into(),
    }
}

/// The Templates panel over `app`, in a window wide enough for its three columns.
fn harness(app: PhotocraftApp) -> Harness<'static, PhotocraftApp> {
    let mut h = Harness::builder().with_size(vec2(900.0, 700.0)).build_ui_state(
        |ui, app: &mut PhotocraftApp| {
            egui::CentralPanel::default().show(ui, |ui| home_screen::templates(app, ui));
        },
        app,
    );
    h.run_steps(2);
    h
}

/// Cards laid out for `category`: one per preset of the category whose name is on a card.
fn cards(h: &Harness<'static, PhotocraftApp>, category: usize) -> usize {
    CATEGORIES.get(category).into_iter().flat_map(|c| c.1).filter(|p| h.query_all_by_label(p.0).count() == 1).count()
}

#[test]
fn chrome_category_defaults_and_round_trips() {
    assert_eq!(ChromeState::default().home_category, 0);
    let json = serde_json::to_string(&ChromeState { home_category: 4, ..Default::default() }).unwrap();
    assert_eq!(serde_json::from_str::<ChromeState>(&json).unwrap().home_category, 4);
}

#[test]
fn a_state_file_without_the_category_still_loads() {
    let loaded: ChromeState = serde_json::from_str(r#"{"statusInfo":"dimensions"}"#).unwrap();
    assert_eq!(loaded.home_category, 0);
}

#[test]
fn every_static_preset_renders_a_card() {
    let mut h = harness(app());
    let mut total = 0;
    for (i, (name, presets)) in CATEGORIES.iter().enumerate().skip(1) {
        h.state_mut().ui.chrome.home_category = i;
        h.run_steps(2);
        assert_eq!(cards(&h, i), presets.len(), "{name}: a card per preset");
        total += presets.len();
    }
    assert_eq!(total, 31, "every preset outside Recent is reachable from the Home screen");
}

#[test]
fn recent_category_shows_recent_documents() {
    // Recent is the dialog's own rule (`new_doc_ui::recent_cards`): the documents created with
    // the dialog first, then the built-in defaults they don't already cover. A card is named
    // after the preset it matches, so a document at a size no preset lists reads "Custom".
    let mut h = harness(app());
    assert_eq!(h.query_all_by_label("Custom").count(), 0, "no document created with the dialog yet");
    assert!(h.query_by_label("Default PhotoCraft Size").is_some(), "the dialog's Recent defaults still show");

    h.state_mut().session.presets.recent_documents = vec![recent(777, 333, 100.0), recent(640, 480, 96.0), recent(321, 654, 90.0)];
    h.run_steps(2);
    assert_eq!(h.query_all_by_label("Custom").count(), 3, "one card per recently created document");
    assert!(h.query_by_label("Default PhotoCraft Size").is_some(), "and the dialog's defaults beside them");
}

#[test]
fn an_out_of_range_category_falls_back_to_recent() {
    // 999 is what a hand-edited state file can leave behind.
    let mut app = app();
    app.ui.chrome.home_category = 999;
    app.session.presets.recent_documents = vec![recent(777, 333, 100.0)];
    let h = harness(app);
    assert!(home_screen::category_presets(999).is_empty(), "an index past the last category has no presets to look up");
    assert!(h.query_by_label("Custom").is_some(), "999 draws Recent, as 0 does");
    assert!(h.query_by_label("Film & Video").is_some(), "the category row still offers every category");
}

#[test]
fn clicking_a_chip_switches_category() {
    let mut h = harness(app());
    h.get_by_label("Web").click();
    h.run_steps(2);
    assert_eq!(h.state().ui.chrome.home_category, 4, "Web is the fifth category");
    assert!(h.query_by_label("MacBook Pro 16\"").is_some(), "and its cards are the ones drawn");
}

#[test]
fn clicking_a_card_opens_new_document_with_that_preset() {
    let mut h = harness(app());
    h.state_mut().ui.chrome.home_category = 6;
    h.run_steps(2);
    h.get_by_label("UHD 4K").click();
    h.run_steps(2);
    let app = h.state();
    let d = app.ui.dialogs.iter().find(|d| d.kind == crate::state::DialogKind::NewDocument).expect("File › New is open");
    let fields = &d.fields;
    assert_eq!((fields["width"].as_u64(), fields["height"].as_u64()), (Some(3840), Some(2160)));
    assert_eq!((fields["resolution"].as_f64(), fields["__preset"].as_str()), (Some(72.0), Some("UHD 4K")));
}
