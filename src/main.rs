use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;
use adw::{
    AboutDialog, Application, ApplicationWindow, ColorScheme, ComboRow, HeaderBar,
    PreferencesGroup, PreferencesDialog, PreferencesPage, ShortcutsDialog, ShortcutsItem,
    ShortcutsSection, StyleManager, SwitchRow,
};
use gtk4::{
    Box, Button, EventControllerKey, MenuButton, Orientation, Paned, PropagationPhase,
    PropertyExpression, ScrolledWindow, StringObject,
};
use gtk4::{gio, Settings};
use pulldown_cmark::{html, Options, Parser};
use sourceview5::{prelude::*, Buffer as SourceBuffer, View as SourceView, VimIMContext};
use webkit6::prelude::*;
use webkit6::WebView;

const PREVIEW_CSS_DARK: &str = r#"
    :root { color-scheme: dark; background: #1a1a1a !important; }
    html { background: #1a1a1a !important; min-height: 100%; }
    body { font-family: 'Cantarell','Inter',system-ui,sans-serif; font-size: 15px; line-height: 1.7;
        padding: 16px 24px; margin: 0; min-height: 100%; color: #e0e0e0; background: #1a1a1a !important; word-wrap: break-word; }
    h1,h2,h3,h4,h5,h6 { color: #fff; margin-top: 1.2em; margin-bottom: 0.4em; font-weight: 600; }
    h1 { font-size: 1.8em; border-bottom: 1px solid #444; padding-bottom: 0.3em; }
    h2 { font-size: 1.5em; border-bottom: 1px solid #3a3a3a; padding-bottom: 0.2em; }
    h3 { font-size: 1.25em; }
    p { margin: 0.6em 0; }
    a { color: #78b9f5; text-decoration: none; }
    a:hover { text-decoration: underline; }
    code { font-family: 'JetBrains Mono','Source Code Pro',monospace; background: #1e1e1e; padding: 2px 6px; border-radius: 4px; font-size: 0.9em; }
    pre { background: #1e1e1e; padding: 14px 18px; border-radius: 8px; overflow-x: auto; border: 1px solid #3a3a3a; }
    pre code { background: none; padding: 0; }
    blockquote { border-left: 3px solid #78b9f5; margin: 0.8em 0; padding: 0.4em 1em; color: #b0b0b0; background: #252525; border-radius: 0 6px 6px 0; }
    ul,ol { padding-left: 1.8em; }
    li { margin: 0.25em 0; }
    hr { border: none; border-top: 1px solid #444; margin: 1.5em 0; }
    table { border-collapse: collapse; width: 100%; margin: 1em 0; }
    th,td { border: 1px solid #444; padding: 8px 12px; text-align: left; }
    th { background: #333; font-weight: 600; }
    img { max-width: 100%; border-radius: 6px; }
    strong { color: #f0f0f0; }
    em { color: #d0d0d0; }
    .placeholder { color: #a8a8a8; text-align: center; margin-top: 2em; }
    .metadata { 
        background: #252525; 
        border: 1px dashed #444; 
        border-radius: 8px; 
        padding: 12px; 
        margin-bottom: 20px; 
        font-family: 'JetBrains Mono', monospace; 
        font-size: 0.85em;
        color: #a0a0a0;
        white-space: pre-wrap;
    }
    .metadata::before {
        content: "metadata";
        display: block;
        font-size: 0.7em;
        color: #666;
        margin-bottom: 6px;
        border-bottom: 1px solid #333;
        padding-bottom: 2px;
    }
"#;

const PRINT_CSS: &str = r#"
    @media print {
        html, body, :root { margin: 0 !important; padding: 0 !important; border: none !important; outline: none !important; }
        body { padding: 16px 24px !important; }
        img { margin: 0 !important; padding: 0 !important; border: none !important; outline: none !important; box-shadow: none !important; }
    }
"#;

const PREVIEW_CSS_LIGHT: &str = r#"
    :root { color-scheme: light; background: #fafafa !important; }
    html { background: #fafafa !important; min-height: 100%; }
    body { font-family: 'Cantarell','Inter',system-ui,sans-serif; font-size: 15px; line-height: 1.7;
        padding: 16px 24px; margin: 0; min-height: 100%; color: #241f31; background: #fafafa !important; word-wrap: break-word; }
    h1,h2,h3,h4,h5,h6 { color: #1c1c1c; margin-top: 1.2em; margin-bottom: 0.4em; font-weight: 600; }
    h1 { font-size: 1.8em; border-bottom: 1px solid #c0bfc4; padding-bottom: 0.3em; }
    h2 { font-size: 1.5em; border-bottom: 1px solid #d1d0d5; padding-bottom: 0.2em; }
    h3 { font-size: 1.25em; }
    p { margin: 0.6em 0; }
    a { color: #1c71d8; text-decoration: none; }
    a:hover { text-decoration: underline; }
    code { font-family: 'JetBrains Mono','Source Code Pro',monospace; background: #ebebeb; padding: 2px 6px; border-radius: 4px; font-size: 0.9em; color: #1c1c1c; }
    pre { background: #ebebeb; padding: 14px 18px; border-radius: 8px; overflow-x: auto; border: 1px solid #d1d0d5; }
    pre code { background: none; padding: 0; }
    blockquote { border-left: 3px solid #1c71d8; margin: 0.8em 0; padding: 0.4em 1em; color: #56565c; background: #f0eff1; border-radius: 0 6px 6px 0; }
    ul,ol { padding-left: 1.8em; }
    li { margin: 0.25em 0; }
    hr { border: none; border-top: 1px solid #c0bfc4; margin: 1.5em 0; }
    table { border-collapse: collapse; width: 100%; margin: 1em 0; }
    th,td { border: 1px solid #c0bfc4; padding: 8px 12px; text-align: left; }
    th { background: #ebe9ed; font-weight: 600; }
    img { max-width: 100%; border-radius: 6px; }
    strong { color: #1c1c1c; }
    em { color: #363536; }
    .placeholder { color: #6b6b6b; text-align: center; margin-top: 2em; }
    .metadata { 
        background: #f0eff1; 
        border: 1px dashed #c0bfc4; 
        border-radius: 8px; 
        padding: 12px; 
        margin-bottom: 20px; 
        font-family: 'JetBrains Mono', monospace; 
        font-size: 0.85em;
        color: #666;
        white-space: pre-wrap;
    }
    .metadata::before {
        content: "metadata";
        display: block;
        font-size: 0.7em;
        color: #999;
        margin-bottom: 6px;
        border-bottom: 1px solid #d1d0d5;
        padding-bottom: 2px;
    }
"#;

const PREF_THEME: &str = "theme";
const PREF_SCHEME: &str = "color-scheme";
const PREF_SYNC_SCROLL: &str = "sync-scroll";
const PREF_DOM_INJECTION: &str = "dom-injection";
const PREF_DEBOUNCE: &str = "debounce-duration";
const PREF_SHOW_METADATA: &str = "metadata-mode";
const PREF_DEFAULT_VIEW: &str = "default-view";
const PREF_READABLE_LINE: &str = "readable-line-length";
const PREF_MAX_WIDTH: &str = "max-content-width";

const DEFAULT_THEME: &str = "default";
const DEFAULT_SCHEME: &str = "Adwaita-dark";
const DEFAULT_SYNC_SCROLL: &str = "true";
const DEFAULT_DOM_INJECTION: &str = "true";
const DEFAULT_DEBOUNCE: &str = "150";
const DEFAULT_SHOW_METADATA: &str = "show";
const DEFAULT_VIEW: &str = "dual-pane";
const DEFAULT_READABLE_LINE: &str = "true";
const DEFAULT_MAX_WIDTH: &str = "1000";

// Keybindings
const PREF_KEY_OPEN: &str = "key-open";
const PREF_KEY_SAVE: &str = "key-save";
const PREF_KEY_SAVE_AS: &str = "key-save-as";
const PREF_KEY_EXPORT_PDF: &str = "key-export-pdf";
const PREF_KEY_PREFS: &str = "key-preferences";
const PREF_KEY_SHORTCUTS: &str = "key-shortcuts-help";
const PREF_KEY_SEARCH: &str = "key-search";
const PREF_KEY_TOGGLE_EDITOR: &str = "key-toggle-editor";
const PREF_KEY_CYCLE_VIEW: &str = "key-cycle-view";
const PREF_KEY_TOGGLE_READABLE: &str = "key-toggle-readable";
const PREF_KEY_QUIT: &str = "key-quit";

const DEFAULT_KEY_OPEN: &str = "<Control>o";
const DEFAULT_KEY_SAVE: &str = "<Control>s";
const DEFAULT_KEY_SAVE_AS: &str = "<Control><Shift>s";
const DEFAULT_KEY_EXPORT_PDF: &str = "<Control>p";
const DEFAULT_KEY_PREFS: &str = "<Control>comma";
const DEFAULT_KEY_SHORTCUTS: &str = "<Control>question";
const DEFAULT_KEY_SEARCH: &str = "<Control>f";
const DEFAULT_KEY_TOGGLE_EDITOR: &str = "<Control>e";
const DEFAULT_KEY_CYCLE_VIEW: &str = "<Control>l";
const DEFAULT_KEY_TOGGLE_READABLE: &str = "<Control>r";
const DEFAULT_KEY_QUIT: &str = "<Control>q";


fn config_path() -> PathBuf {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    config.join("MarkView").join("preferences.ini")
}

fn load_pref(key: &str, default: &str) -> String {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix(&format!("{}=", key)) {
                    return rest.to_string();
                }
            }
        }
    }
    default.to_string()
}

fn save_pref(key: &str, value: &str) {
    let path = config_path();
    let parent = path.parent().unwrap();
    let _ = std::fs::create_dir_all(parent);
    let mut prefs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    prefs.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
    }
    prefs.insert(key.to_string(), value.to_string());
    let content: String = prefs
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");
    let mut final_content = content;
    if !final_content.is_empty() {
        final_content.push('\n');
    }
    let _ = std::fs::write(&path, final_content);
}

fn build_html_page(body: &str, dark: bool) -> String {
    let css = if dark { PREVIEW_CSS_DARK } else { PREVIEW_CSS_LIGHT };
    
    let is_readable = load_pref(PREF_READABLE_LINE, DEFAULT_READABLE_LINE) == "true";
    let max_width = load_pref(PREF_MAX_WIDTH, DEFAULT_MAX_WIDTH);
    
    let readable_css = if is_readable {
        format!("body {{ max-width: {}px; margin-left: auto !important; margin-right: auto !important; }}", max_width)
    } else {
        "".to_string()
    };

    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>{} {} {}</style></head><body>{}</body></html>",
        css, PRINT_CSS, readable_css, body
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_javascript_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$")
}

fn base_uri_for_preview(current_file: Option<&gio::File>) -> Option<String> {
    if let Some(file) = current_file {
        if let Some(parent) = file.parent() {
            let mut uri = parent.uri().to_string();
            if !uri.ends_with('/') {
                uri.push('/');
            }
            return Some(uri);
        }
    }
    std::env::current_dir()
        .ok()
        .and_then(|path| path.canonicalize().ok())
        .map(|path| format!("file://{}/", path.to_string_lossy()))
}

fn create_md_filters() -> gio::ListStore {
    let md = gtk4::FileFilter::new();
    md.add_pattern("*.md");
    md.add_pattern("*.markdown");
    md.set_name(Some("Markdown Files"));
    let all = gtk4::FileFilter::new();
    all.add_pattern("*");
    all.set_name(Some("All Files"));
    let filters = gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&md);
    filters.append(&all);
    filters
}

fn create_pdf_filters() -> gio::ListStore {
    let pdf = gtk4::FileFilter::new();
    pdf.add_mime_type("application/pdf");
    pdf.add_pattern("*.pdf");
    pdf.set_name(Some("PDF"));
    let all = gtk4::FileFilter::new();
    all.add_pattern("*");
    all.set_name(Some("All Files"));
    let filters = gio::ListStore::new::<gtk4::FileFilter>();
    filters.append(&pdf);
    filters.append(&all);
    filters
}

fn apply_accels(app: &Application) {
    app.set_accels_for_action("app.open", &[&load_pref(PREF_KEY_OPEN, DEFAULT_KEY_OPEN)]);
    app.set_accels_for_action("app.save", &[&load_pref(PREF_KEY_SAVE, DEFAULT_KEY_SAVE)]);
    app.set_accels_for_action("app.save-as", &[&load_pref(PREF_KEY_SAVE_AS, DEFAULT_KEY_SAVE_AS)]);
    app.set_accels_for_action("app.export-pdf", &[&load_pref(PREF_KEY_EXPORT_PDF, DEFAULT_KEY_EXPORT_PDF)]);
    app.set_accels_for_action("app.preferences", &[&load_pref(PREF_KEY_PREFS, DEFAULT_KEY_PREFS)]);
    app.set_accels_for_action("app.shortcuts", &[&load_pref(PREF_KEY_SHORTCUTS, DEFAULT_KEY_SHORTCUTS)]);
    app.set_accels_for_action("app.search", &[&load_pref(PREF_KEY_SEARCH, DEFAULT_KEY_SEARCH)]);
    app.set_accels_for_action("app.toggle-editor", &[&load_pref(PREF_KEY_TOGGLE_EDITOR, DEFAULT_KEY_TOGGLE_EDITOR)]);
    app.set_accels_for_action("app.cycle-view", &[&load_pref(PREF_KEY_CYCLE_VIEW, DEFAULT_KEY_CYCLE_VIEW)]);
    app.set_accels_for_action("app.quit", &[&load_pref(PREF_KEY_QUIT, DEFAULT_KEY_QUIT)]);
}

fn build_ui(app: &Application, initial_file: Option<gio::File>) {
    let settings = Settings::default().expect("Failed to get default settings");
    settings.set_gtk_keynav_use_caret(false);
    settings.set_gtk_error_bell(false);

    let current_file: Rc<RefCell<Option<gio::File>>> = Rc::new(RefCell::new(initial_file.clone()));
    let vim_controller: Rc<RefCell<Option<EventControllerKey>>> =
        Rc::new(RefCell::new(None));

    // --- Header Bar ---
    let header_bar = HeaderBar::new();

    let open_button = Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text("Open (Ctrl+O)")
        .action_name("app.open")
        .build();

    let save_button = Button::builder()
        .icon_name("media-floppy-symbolic")
        .tooltip_text("Save (Ctrl+S)")
        .action_name("app.save")
        .build();

    let export_pdf_button = Button::builder()
        .icon_name("document-save-symbolic")
        .tooltip_text("Export as PDF")
        .action_name("app.export-pdf")
        .build();

    let menu_button = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .build();

    let sidebar_toggle = Button::builder()
        .icon_name("view-dual-symbolic")
        .tooltip_text("Hide left panel")
        .build();

    header_bar.pack_start(&open_button);
    header_bar.pack_start(&sidebar_toggle);
    // pack_end adds right-to-left, so menu first, then pdf, then save
    header_bar.pack_end(&menu_button);
    header_bar.pack_end(&export_pdf_button);
    header_bar.pack_end(&save_button);

    // --- Editor (left) ---
    let paned = Paned::builder()
        .orientation(Orientation::Horizontal)
        .vexpand(true)
        .hexpand(true)
        .build();

    let source_view = SourceView::new();
    let source_buffer: SourceBuffer = source_view.buffer().downcast().unwrap();
    source_buffer.set_language(Some(
        &sourceview5::LanguageManager::default()
            .language("markdown")
            .unwrap(),
    ));
    let scheme_mgr = sourceview5::StyleSchemeManager::default();
    scheme_mgr.append_search_path("data/styles");

    let scheme_id = load_pref(PREF_SCHEME, DEFAULT_SCHEME);
    if let Some(scheme) = scheme_mgr.scheme(&scheme_id) {
        source_buffer.set_style_scheme(Some(&scheme));
    } else if let Some(scheme) = scheme_mgr.scheme(DEFAULT_SCHEME) {
        source_buffer.set_style_scheme(Some(&scheme));
    }

    let style_mgr = StyleManager::default();
    match load_pref(PREF_THEME, DEFAULT_THEME).as_str() {
        "force-dark" => style_mgr.set_color_scheme(ColorScheme::ForceDark),
        "force-light" => style_mgr.set_color_scheme(ColorScheme::ForceLight),
        _ => style_mgr.set_color_scheme(ColorScheme::Default),
    }
    source_buffer.set_highlight_syntax(true);
    // Add a special tag for the current search match
    source_buffer.tag_table().add(&gtk4::TextTag::builder()
        .name("active-match")
        .background("#f5d67b")
        .foreground("#000000")
        .build());
    source_view.set_show_line_numbers(true);
    source_view.set_monospace(true);
    source_view.set_tab_width(4);
    source_view.set_auto_indent(true);
    source_view.set_indent_on_tab(true);
    source_view.set_smart_backspace(true);
    source_view.set_wrap_mode(gtk4::WrapMode::Word);
    source_view.set_top_margin(8);
    source_view.set_bottom_margin(8);
    source_view.set_left_margin(8);
    source_view.set_right_margin(8);

    let editor_scroll = ScrolledWindow::builder()
        .child(&source_view)
        .vexpand(true)
        .hexpand(true)
        .build();

    // --- Preview (right) ---
    let webview = WebView::new();
    webview.set_vexpand(true);
    webview.set_hexpand(true);
    let base_uri_initial = base_uri_for_preview(None);
    let dark = StyleManager::default().is_dark();
    webview.load_html(
        &build_html_page("<p class='placeholder'>Start typing markdown on the left…</p>", dark),
        base_uri_initial.as_deref(),
    );
    let (r, g, b) = if dark { (0.102, 0.102, 0.102) } else { (0.98, 0.98, 0.98) };
    webview.set_background_color(&gtk4::gdk::RGBA::new(r, g, b, 1.0));

    let preview_scroll = ScrolledWindow::builder()
        .child(&webview)
        .vexpand(true)
        .hexpand(true)
        .build();

    // --- Sync Scroll ---
    {
        let editor_adj = editor_scroll.vadjustment();
        let wv = webview.clone();
        
        let sync_scroll = {
            let editor_adj = editor_adj.clone();
            let wv = wv.clone();
            move || {
                let value = editor_adj.value();
                let upper = editor_adj.upper();
                let page_size = editor_adj.page_size();
                if upper > page_size {
                    let percent = value / (upper - page_size);
                    let script = format!(
                        "window.scrollTo(0, (document.documentElement.scrollHeight - window.innerHeight) * {});",
                        percent
                    );
                    wv.evaluate_javascript(&script, None, None, None::<&gio::Cancellable>, |_| {});
                }
            }
        };

        editor_adj.connect_value_changed({
            let sync_scroll = sync_scroll.clone();
            move |_| {
                if load_pref(PREF_SYNC_SCROLL, DEFAULT_SYNC_SCROLL) == "true" {
                    sync_scroll();
                }
            }
        });

        wv.connect_load_changed({
            let editor_adj = editor_adj.clone();
            move |wv, event| {
                if event == webkit6::LoadEvent::Finished
                    && load_pref(PREF_SYNC_SCROLL, DEFAULT_SYNC_SCROLL) == "true"
                {
                    let value = editor_adj.value();
                    let upper = editor_adj.upper();
                    let page_size = editor_adj.page_size();
                    if upper > page_size {
                        let percent = value / (upper - page_size);
                        let script = format!(
                            "window.scrollTo(0, (document.documentElement.scrollHeight - window.innerHeight) * {});",
                            percent
                        );
                        wv.evaluate_javascript(&script, None, None, None::<&gio::Cancellable>, |_| {});
                    }
                }
            }
        });
    }

    // --- Search & Replace UI ---
    let search_bar = gtk4::SearchBar::builder()
        .key_capture_widget(&source_view)
        .build();
    
    // Style the search group for a floating look
    let search_group = Box::new(Orientation::Horizontal, 10);
    search_group.add_css_class("card");
    
    let provider = gtk4::CssProvider::new();
    provider.load_from_data("
        searchbar, revealer, searchbar > revealer > box { 
            background-color: transparent; 
            border-style: none; 
            box-shadow: none; 
        }
        .card { 
            border-radius: 12px; 
            padding: 8px; 
            border: 1px solid alpha(@window_fg_color, 0.1); 
            background-color: @window_bg_color;
            box-shadow: 0 4px 16px rgba(0,0,0,0.3);
        }
    ");
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    search_group.set_margin_bottom(6);
    search_group.set_margin_start(6);
    search_group.set_margin_end(6);
    search_group.set_halign(gtk4::Align::Center);
    
    // Left side: Vertical entries
    let entries_vbox = Box::new(Orientation::Vertical, 4);
    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text("Find...")
        .width_request(250)
        .build();
    let replace_entry = gtk4::Entry::builder()
        .placeholder_text("Replace with...")
        .width_request(250)
        .build();
    entries_vbox.append(&search_entry);
    entries_vbox.append(&replace_entry);

    // Right side: Horizontal options
    let options_hbox = Box::new(Orientation::Horizontal, 4);
    options_hbox.set_valign(gtk4::Align::Center);
    
    let regex_toggle = gtk4::ToggleButton::builder()
        .label(".*")
        .tooltip_text("Use Regular Expressions")
        .build();
    let prev_button = Button::builder().icon_name("go-up-symbolic").tooltip_text("Previous").build();
    let next_button = Button::builder().icon_name("go-down-symbolic").tooltip_text("Next").build();
    let replace_button = Button::builder().label("Replace").build();
    let replace_all_button = Button::builder().label("Replace All").build();
    let close_button = Button::builder().icon_name("window-close-symbolic").tooltip_text("Close").build();

    options_hbox.append(&regex_toggle);
    options_hbox.append(&prev_button);
    options_hbox.append(&next_button);
    options_hbox.append(&replace_button);
    options_hbox.append(&replace_all_button);
    options_hbox.append(&close_button);

    search_group.append(&entries_vbox);
    search_group.append(&options_hbox);
    
    search_bar.set_child(Some(&search_group));
    search_bar.connect_entry(&search_entry);

    {
        let sb = search_bar.clone();
        close_button.connect_clicked(move |_| {
            sb.set_property("search-mode-enabled", false);
        });
    }

    // --- Search Logic ---
    let search_settings = sourceview5::SearchSettings::new();
    let search_context = sourceview5::SearchContext::new(&source_buffer, Some(&search_settings));
    search_context.set_highlight(true);

    {
        let settings = search_settings.clone();
        let buf = source_buffer.clone();
        search_entry.connect_search_changed(move |entry| {
            settings.set_search_text(Some(&entry.text()));
            let (start, end) = buf.bounds();
            buf.remove_tag_by_name("active-match", &start, &end);
        });
    }

    {
        let settings = search_settings.clone();
        regex_toggle.connect_toggled(move |btn| {
            settings.set_regex_enabled(btn.is_active());
        });
    }

    {
        let context = search_context.clone();
        let sv = source_view.clone();
        next_button.connect_clicked(move |_| {
            let buf = sv.buffer();
            let mark = buf.mark("insert").unwrap();
            let iter = buf.iter_at_mark(&mark);

            if let Some((start, end, _)) = context.forward(&iter) {
                let (b_start, b_end) = buf.bounds();
                buf.remove_tag_by_name("active-match", &b_start, &b_end);
                buf.apply_tag_by_name("active-match", &start, &end);

                buf.select_range(&end, &start);
                sv.grab_focus();

                // Use a fresh iter for scrolling to avoid tree mismatch criticals
                let mut fresh_iter = buf.iter_at_offset(start.offset());
                sv.scroll_to_iter(&mut fresh_iter, 0.0, true, 0.5, 0.5);
            }
        });
    }

    {
        let context = search_context.clone();
        let sv = source_view.clone();
        prev_button.connect_clicked(move |_| {
            let buf = sv.buffer();
            let mark = buf.mark("insert").unwrap();
            let iter = buf.iter_at_mark(&mark);

            if let Some((start, end, _)) = context.backward(&iter) {
                let (b_start, b_end) = buf.bounds();
                buf.remove_tag_by_name("active-match", &b_start, &b_end);
                buf.apply_tag_by_name("active-match", &start, &end);

                buf.select_range(&start, &end);
                sv.grab_focus();

                // Use a fresh iter for scrolling to avoid tree mismatch criticals
                let mut fresh_iter = buf.iter_at_offset(start.offset());
                sv.scroll_to_iter(&mut fresh_iter, 0.0, true, 0.5, 0.5);
            }
        });
    }

    {
        let context = search_context.clone();
        let re = replace_entry.clone();
        replace_button.connect_clicked(move |_| {
            let text = re.text().to_string();
            let buf = context.buffer().downcast::<SourceBuffer>().unwrap();
            if let Some((start, end)) = buf.selection_bounds() {
                let mut start_mut = start;
                let mut end_mut = end;
                let _ = context.replace(&mut start_mut, &mut end_mut, &text);
            }
        });
    }

    {
        let context = search_context.clone();
        let re = replace_entry.clone();
        replace_all_button.connect_clicked(move |_| {
            let text = re.text().to_string();
            let _ = context.replace_all(&text);
        });
    }

    let main_overlay = gtk4::Overlay::builder()
        .child(&paned)
        .build();
    main_overlay.add_overlay(&search_bar);
    search_bar.set_valign(gtk4::Align::Start);
    search_bar.set_halign(gtk4::Align::Center);

    paned.set_start_child(Some(&editor_scroll));
    paned.set_end_child(Some(&preview_scroll));
    paned.set_shrink_start_child(true);
    paned.set_position(400);

    let saved_paned_pos: Rc<RefCell<i32>> = Rc::new(RefCell::new(400));
    let left_panel_visible: Rc<RefCell<bool>> = Rc::new(RefCell::new(true));

    // Apply Default View Mode
    {
        let saved_view = load_pref(PREF_DEFAULT_VIEW, DEFAULT_VIEW);
        match saved_view.as_str() {
            "preview-only" => {
                paned.set_position(0);
                sidebar_toggle.set_icon_name("sidebar-show-symbolic");
                sidebar_toggle.set_tooltip_text(Some("Show left panel"));
                *left_panel_visible.borrow_mut() = false;
            }
            "editor-only" => {
            }
            _ => {}
        }
    }

    // --- Window ---
    let content = Box::new(Orientation::Vertical, 0);
    content.append(&header_bar);
    content.append(&main_overlay);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("MarkView")
        .default_width(1100)
        .default_height(700)
        .content(&content)
        .build();

    {
        let paned = paned.clone();
        let saved_pos = saved_paned_pos.clone();
        let visible = left_panel_visible.clone();
        sidebar_toggle.connect_clicked(move |btn| {
            if *visible.borrow() {
                let pos = paned.position();
                *saved_pos.borrow_mut() = if pos > 0 { pos } else { 400 };
                paned.set_position(0);
                btn.set_icon_name("sidebar-show-symbolic");
                btn.set_tooltip_text(Some("Show left panel"));
                *visible.borrow_mut() = false;
            } else {
                paned.set_position(*saved_pos.borrow());
                btn.set_icon_name("view-dual-symbolic");
                btn.set_tooltip_text(Some("Hide left panel"));
                *visible.borrow_mut() = true;
            }
        });
    }

    // --- Live Preview ---
    let pending_update: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    let is_first_render = Rc::new(RefCell::new(true));

    let refresh_preview = {
        let wv = webview.clone();
        let cf_preview = current_file.clone();
        let is_first = is_first_render.clone();
        move |buffer: &SourceBuffer, force_full_reload: bool| {
            let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            
            let metadata_mode = load_pref(PREF_SHOW_METADATA, DEFAULT_SHOW_METADATA);
            let mut options = Options::all();
            
            // "ignore" means we don't want the parser to treat it as special metadata
            if metadata_mode == "ignore" {
                options.remove(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
            }

            let parser = Parser::new_ext(&text, options);
            let mut html_out = String::new();
            
            if metadata_mode == "show" || metadata_mode == "hide" {
                let mut events = Vec::new();
                let mut in_metadata = false;
                for event in parser {
                    match event {
                        pulldown_cmark::Event::Start(pulldown_cmark::Tag::MetadataBlock(_)) => {
                            in_metadata = true;
                            if metadata_mode == "show" {
                                events.push(pulldown_cmark::Event::Html("<div class=\"metadata\">".into()));
                            }
                        }
                        pulldown_cmark::Event::End(pulldown_cmark::TagEnd::MetadataBlock(_)) => {
                            in_metadata = false;
                            if metadata_mode == "show" {
                                events.push(pulldown_cmark::Event::Html("</div>".into()));
                            }
                        }
                        pulldown_cmark::Event::Text(t) => {
                            if in_metadata {
                                if metadata_mode == "show" {
                                    events.push(pulldown_cmark::Event::Html(html_escape(&t).into()));
                                }
                            } else {
                                events.push(pulldown_cmark::Event::Text(t));
                            }
                        }
                        _ => {
                            if !in_metadata {
                                events.push(event);
                            }
                        }
                    }
                }
                html::push_html(&mut html_out, events.into_iter());
            } else {
                html::push_html(&mut html_out, parser);
            }

            let dom_injection = load_pref(PREF_DOM_INJECTION, DEFAULT_DOM_INJECTION) == "true";
            let dark = StyleManager::default().is_dark();
            let first = *is_first.borrow();

            if dom_injection && !first && !force_full_reload {
                let escaped_html = escape_javascript_string(&html_out);
                let js = format!("document.body.innerHTML = `{}`;", escaped_html);
                wv.evaluate_javascript(&js, None, None, None::<&gio::Cancellable>, |_| {});
            } else {
                let base_uri = base_uri_for_preview(cf_preview.borrow().as_ref());
                let body = if html_out.is_empty() {
                    "<p class='placeholder'>Start typing markdown on the left…</p>".to_string()
                } else {
                    html_out
                };
                wv.load_html(&build_html_page(&body, dark), base_uri.as_deref());
                let (r, g, b) = if dark { (0.102, 0.102, 0.102) } else { (0.98, 0.98, 0.98) };
                wv.set_background_color(&gtk4::gdk::RGBA::new(r, g, b, 1.0));
                *is_first.borrow_mut() = false;
            }
        }
    };

    source_buffer.connect_changed({
        let refresh_preview = refresh_preview.clone();
        let pending_update = pending_update.clone();
        move |buffer| {
            if let Some(source_id) = pending_update.borrow_mut().take() {
                source_id.remove();
            }

            let debounce_ms = load_pref(PREF_DEBOUNCE, DEFAULT_DEBOUNCE)
                .parse::<f64>()
                .unwrap_or(150.0) as u32;

            if debounce_ms == 0 {
                refresh_preview(buffer, false);
            } else {
                let buffer = buffer.clone();
                let refresh_preview = refresh_preview.clone();
                let pending_update_inner = pending_update.clone();

                let source_id = glib::timeout_add_local(
                    std::time::Duration::from_millis(debounce_ms as u64),
                    move || {
                        refresh_preview(&buffer, false);
                        *pending_update_inner.borrow_mut() = None;
                        glib::ControlFlow::Break
                    },
                );
                *pending_update.borrow_mut() = Some(source_id);
            }
        }
    });

    StyleManager::default().connect_dark_notify({
        let wv = webview.clone();
        let sb = source_buffer.clone();
        let cf = current_file.clone();
        move |_| {
            let text = sb.text(&sb.start_iter(), &sb.end_iter(), false);
            let parser = Parser::new_ext(&text, Options::all());
            let mut html_out = String::new();
            html::push_html(&mut html_out, parser);
            let base_uri = base_uri_for_preview(cf.borrow().as_ref());
            let dark = StyleManager::default().is_dark();
            let body = if html_out.is_empty() {
                "<p class='placeholder'>Start typing markdown on the left…</p>".to_string()
            } else {
                html_out
            };
            wv.load_html(&build_html_page(&body, dark), base_uri.as_deref());
            let (r, g, b) = if dark { (0.102, 0.102, 0.102) } else { (0.98, 0.98, 0.98) };
            wv.set_background_color(&gtk4::gdk::RGBA::new(r, g, b, 1.0));
        }
    });

    // --- Menu ---
    let menu = gio::Menu::new();
    let file_sec = gio::Menu::new();
    file_sec.append(Some("Open…"), Some("app.open"));
    file_sec.append(Some("Save As…"), Some("app.save-as"));
    file_sec.append(Some("Export as PDF…"), Some("app.export-pdf"));
    menu.append_section(None, &file_sec);
    let app_sec = gio::Menu::new();
    app_sec.append(Some("Preferences"), Some("app.preferences"));
    app_sec.append(Some("Keyboard Shortcuts"), Some("app.shortcuts"));
    app_sec.append(Some("About"), Some("app.about"));
    app_sec.append(Some("Quit"), Some("app.quit"));
    menu.append_section(None, &app_sec);
    menu_button.set_menu_model(Some(&menu));

    // === Actions ===

    // Open
    let open_action = gio::SimpleAction::new("open", None);
    {
        let w = window.clone();
        let buf = source_buffer.clone();
        let cf = current_file.clone();
        let is_first = is_first_render.clone();
        open_action.connect_activate(move |_, _| {
            let dialog = gtk4::FileDialog::builder()
                .title("Open Markdown File")
                .build();
            dialog.set_filters(Some(&create_md_filters()));
            let buf = buf.clone();
            let cf = cf.clone();
            let is_first = is_first.clone();
            let w = w.clone();
            let w_inner = w.clone();
            dialog.open(Some(&w), None::<&gio::Cancellable>, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                *cf.borrow_mut() = Some(file);
                                *is_first.borrow_mut() = true;
                                buf.set_text(&content);
                                if let Some(name) = path.file_name() {
                                    w_inner.set_title(Some(&format!("{} — MarkView", name.to_string_lossy())));
                                }
                            }
                            Err(e) => eprintln!("Failed to read file: {e}"),
                        }
                    }
                }
            });
        });
    }
    app.add_action(&open_action);

    // Save
    let save_action = gio::SimpleAction::new("save", None);
    {
        let w = window.clone();
        let buf = source_buffer.clone();
        let cf = current_file.clone();
        save_action.connect_activate(move |_, _| {
            let file_opt = cf.borrow().clone();
            if let Some(file) = file_opt {
                if let Some(path) = file.path() {
                    let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                    if let Err(e) = std::fs::write(&path, text.as_str()) {
                        eprintln!("Failed to save: {e}");
                    }
                }
            } else {
                // No file yet — show Save As dialog
                let dialog = gtk4::FileDialog::builder()
                    .title("Save Markdown File")
                    .initial_name("untitled.md")
                    .build();
                dialog.set_filters(Some(&create_md_filters()));
                let buf = buf.clone();
                let cf = cf.clone();
                let w = w.clone();
                let w_inner = w.clone();
                dialog.save(Some(&w), None::<&gio::Cancellable>, move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                            match std::fs::write(&path, text.as_str()) {
                                Ok(_) => {
                                    if let Some(name) = path.file_name() {
                                        w_inner.set_title(Some(&format!("{} — MarkView", name.to_string_lossy())));
                                    }
                                    *cf.borrow_mut() = Some(file);
                                }
                                Err(e) => eprintln!("Failed to save: {e}"),
                            }
                        }
                    }
                });
            }
        });
    }
    app.add_action(&save_action);

    // Save As
    let save_as_action = gio::SimpleAction::new("save-as", None);
    {
        let w = window.clone();
        let buf = source_buffer.clone();
        let cf = current_file.clone();
        save_as_action.connect_activate(move |_, _| {
            let current = cf.borrow().clone();
            let dialog = if let Some(ref f) = current {
                gtk4::FileDialog::builder()
                    .title("Save Markdown File")
                    .initial_file(f)
                    .build()
            } else {
                gtk4::FileDialog::builder()
                    .title("Save Markdown File")
                    .initial_name("untitled.md")
                    .build()
            };
            dialog.set_filters(Some(&create_md_filters()));
            let buf = buf.clone();
            let cf = cf.clone();
            let w = w.clone();
            let w_inner = w.clone();
            dialog.save(Some(&w), None::<&gio::Cancellable>, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
                        match std::fs::write(&path, text.as_str()) {
                            Ok(_) => {
                                if let Some(name) = path.file_name() {
                                    w_inner.set_title(Some(&format!("{} — MarkView", name.to_string_lossy())));
                                }
                                *cf.borrow_mut() = Some(file);
                            }
                            Err(e) => eprintln!("Failed to save: {e}"),
                        }
                    }
                }
            });
        });
    }
    app.add_action(&save_as_action);

    // Export PDF
    let export_pdf_action = gio::SimpleAction::new("export-pdf", None);
    {
        let wv = webview.clone();
        let w = window.clone();
        export_pdf_action.connect_activate(move |_, _| {
            let dialog = gtk4::FileDialog::builder()
                .title("Export as PDF")
                .initial_name("document.pdf")
                .build();
            dialog.set_filters(Some(&create_pdf_filters()));
            let wv = wv.clone();
            let w = w.clone();
            let w_parent = w.clone();
            dialog.save(Some(&w_parent), None::<&gio::Cancellable>, move |result| {
                if let Ok(file) = result {
                    let uri = file.uri().to_string();
                    let settings = gtk4::PrintSettings::new();
                    settings.set(gtk4::PRINT_SETTINGS_OUTPUT_URI.as_str(), Some(uri.as_str()));
                    settings.set(
                        gtk4::PRINT_SETTINGS_OUTPUT_FILE_FORMAT.as_str(),
                        Some("PDF"),
                    );
                    let page_setup = gtk4::PageSetup::new();
                    page_setup.set_top_margin(0.0, gtk4::Unit::Mm);
                    page_setup.set_bottom_margin(0.0, gtk4::Unit::Mm);
                    page_setup.set_left_margin(0.0, gtk4::Unit::Mm);
                    page_setup.set_right_margin(0.0, gtk4::Unit::Mm);
                    let print_op = webkit6::PrintOperation::new(&wv);
                    print_op.set_print_settings(&settings);
                    print_op.set_page_setup(&page_setup);
                    print_op.run_dialog(Some(&w));
                }
            });
        });
    }
    app.add_action(&export_pdf_action);

    // Toggle Editor
    let toggle_editor_action = gio::SimpleAction::new("toggle-editor", None);
    {
        let paned = paned.clone();
        let sidebar_toggle = sidebar_toggle.clone();
        let left_panel_visible = left_panel_visible.clone();
        let saved_paned_pos = saved_paned_pos.clone();
        toggle_editor_action.connect_activate(move |_, _| {
            let is_visible = *left_panel_visible.borrow();
            if is_visible {
                *saved_paned_pos.borrow_mut() = paned.position();
                paned.set_position(0);
                sidebar_toggle.set_icon_name("sidebar-show-symbolic");
                sidebar_toggle.set_tooltip_text(Some("Show left panel"));
                *left_panel_visible.borrow_mut() = false;
            } else {
                paned.set_position(*saved_paned_pos.borrow());
                sidebar_toggle.set_icon_name("view-dual-symbolic");
                sidebar_toggle.set_tooltip_text(Some("Hide left panel"));
                *left_panel_visible.borrow_mut() = true;
            }
        });
    }
    app.add_action(&toggle_editor_action);

    // Toggle Readable Line Length
    let toggle_readable_action = gio::SimpleAction::new("toggle-readable", None);
    {
        let sb = source_buffer.clone();
        let refresh_preview = refresh_preview.clone();
        toggle_readable_action.connect_activate(move |_, _| {
            let current = load_pref(PREF_READABLE_LINE, DEFAULT_READABLE_LINE) == "true";
            save_pref(PREF_READABLE_LINE, if current { "false" } else { "true" });
            refresh_preview(&sb, true);
        });
    }
    app.add_action(&toggle_readable_action);

    // Cycle View Modes
    let cycle_view_action = gio::SimpleAction::new("cycle-view", None);
    {
        let paned = paned.clone();
        let sidebar_toggle = sidebar_toggle.clone();
        let left_panel_visible = left_panel_visible.clone();
        let saved_paned_pos = saved_paned_pos.clone();
        cycle_view_action.connect_activate(move |_, _| {
            let current_pos = paned.position();
            if current_pos == 0 {
                // Currently Preview Only -> Switch back to Dual Pane
                paned.set_position(*saved_paned_pos.borrow());
                *left_panel_visible.borrow_mut() = true;
                sidebar_toggle.set_icon_name("view-dual-symbolic");
            } else {
                // Currently Dual Pane -> Switch to Preview Only
                *saved_paned_pos.borrow_mut() = current_pos;
                paned.set_position(0);
                *left_panel_visible.borrow_mut() = false;
                sidebar_toggle.set_icon_name("sidebar-show-symbolic");
            }
        });
    }
    app.add_action(&cycle_view_action);

    // Preferences
    let preferences_action = gio::SimpleAction::new("preferences", None);
    {
        let w = window.clone();
        let sv = source_view.clone();
        let sb_pref = source_buffer.clone();
        let vc = vim_controller.clone();
        let app = app.clone();
        let refresh_preview = refresh_preview.clone();
        preferences_action.connect_activate(move |_, _| {
            let theme_model = gio::ListStore::new::<StringObject>();
            theme_model.append(&StringObject::new("Auto"));
            theme_model.append(&StringObject::new("Dark"));
            theme_model.append(&StringObject::new("Light"));
            let theme_expr = PropertyExpression::new(StringObject::static_type(), None::<&gtk4::Expression>, "string");
            let theme_row = ComboRow::builder()
                .title("Theme")
                .subtitle("Application appearance")
                .model(&theme_model)
                .expression(&theme_expr)
                .build();
            let saved_theme = load_pref(PREF_THEME, DEFAULT_THEME);
            theme_row.set_selected(match saved_theme.as_str() {
                "force-dark" => 1,
                "force-light" => 2,
                _ => 0,
            });
            let style_mgr = StyleManager::default();
            theme_row.connect_selected_notify({
                let style_mgr = style_mgr.clone();
                move |row| {
                    let scheme = match row.selected() {
                        1 => ColorScheme::ForceDark,
                        2 => ColorScheme::ForceLight,
                        _ => ColorScheme::Default,
                    };
                    style_mgr.set_color_scheme(scheme);
                    save_pref(
                        PREF_THEME,
                        match scheme {
                            ColorScheme::ForceDark => "force-dark",
                            ColorScheme::ForceLight => "force-light",
                            _ => "default",
                        },
                    );
                }
            });

            let scheme_mgr = sourceview5::StyleSchemeManager::default();
            scheme_mgr.append_search_path("data/styles");
            let mut all_ids: Vec<_> = scheme_mgr.scheme_ids();
            all_ids.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            let scheme_model = gio::ListStore::new::<StringObject>();
            let mut scheme_ids = Vec::new();
            for id in all_ids.iter() {
                if let Some(scheme) = scheme_mgr.scheme(id.as_str()) {
                    scheme_model.append(&StringObject::new(&scheme.name().to_string()));
                    scheme_ids.push(id.to_string());
                }
            }
            if scheme_ids.is_empty() {
                for id in all_ids.iter().take(10) {
                    if let Some(scheme) = scheme_mgr.scheme(id.as_str()) {
                        scheme_model.append(&StringObject::new(&scheme.name().to_string()));
                        scheme_ids.push(id.to_string());
                    }
                }
            }
            if scheme_ids.is_empty() {
                scheme_model.append(&StringObject::new("Adwaita dark"));
                scheme_ids.push("Adwaita-dark".to_string());
            }
            let scheme_expr = PropertyExpression::new(StringObject::static_type(), None::<&gtk4::Expression>, "string");
            let scheme_row = ComboRow::builder()
                .title("Editor color scheme")
                .subtitle("Syntax highlighting theme")
                .model(&scheme_model)
                .expression(&scheme_expr)
                .build();
            let saved_scheme = load_pref(PREF_SCHEME, DEFAULT_SCHEME);
            let scheme_idx = scheme_ids.iter().position(|s| s == &saved_scheme).unwrap_or(0);
            scheme_row.set_selected(scheme_idx as u32);
            let scheme_ids = Rc::new(scheme_ids);
            scheme_row.connect_selected_notify({
                let sb = sb_pref.clone();
                let scheme_ids = scheme_ids.clone();
                move |row| {
                    let idx = row.selected() as usize;
                    if let Some(id) = scheme_ids.get(idx) {
                        let mgr = sourceview5::StyleSchemeManager::default();
                        if let Some(scheme) = mgr.scheme(id) {
                            sb.set_style_scheme(Some(&scheme));
                            save_pref(PREF_SCHEME, id);
                        }
                    }
                }
            });

            let appearance_group = PreferencesGroup::new();
            appearance_group.set_title("Appearance");
            appearance_group.add(&theme_row);
            appearance_group.add(&scheme_row);
            let appearance_page = PreferencesPage::builder()
                .title("Appearance")
                .icon_name("preferences-desktop-theme-symbolic")
                .build();
            appearance_page.add(&appearance_group);

            let vim_row = SwitchRow::builder()
                .title("Vim keybindings")
                .subtitle("Use Vim-style keybindings in the editor")
                .active(false)
                .build();
            let line_numbers_row = SwitchRow::builder()
                .title("Show line numbers")
                .subtitle("Display line numbers in the gutter")
                .active(true)
                .build();
            let word_wrap_row = SwitchRow::builder()
                .title("Word wrap")
                .subtitle("Wrap long lines at word boundaries")
                .active(true)
                .build();
            vim_row.set_active(vc.borrow().is_some());
            line_numbers_row.set_active(sv.shows_line_numbers());
            word_wrap_row.set_active(sv.wrap_mode() == gtk4::WrapMode::Word);
            vim_row.connect_active_notify({
                let sv = sv.clone();
                let vc = vc.clone();
                move |row| {
                    if row.is_active() {
                        let vim_ctx = VimIMContext::new();
                        vim_ctx.set_client_widget(Some(&sv));
                        let key_ctrl = EventControllerKey::new();
                        key_ctrl.set_propagation_phase(PropagationPhase::Capture);
                        key_ctrl.set_im_context(Some(&vim_ctx));
                        let ctrl_clone = key_ctrl.clone();
                        sv.add_controller(ctrl_clone);
                        *vc.borrow_mut() = Some(key_ctrl);
                    } else if let Some(ref ctrl) = *vc.borrow() {
                        sv.remove_controller(ctrl);
                        *vc.borrow_mut() = None;
                    }
                }
            });
            line_numbers_row.connect_active_notify({
                let sv = sv.clone();
                let refresh_preview = refresh_preview.clone();
                let sb = sb_pref.clone();
                move |row| {
                    sv.set_show_line_numbers(row.is_active());
                    refresh_preview(&sb, true);
                }
            });
            word_wrap_row.connect_active_notify({
                let sv = sv.clone();
                move |row| {
                    sv.set_wrap_mode(if row.is_active() {
                        gtk4::WrapMode::Word
                    } else {
                        gtk4::WrapMode::None
                    });
                }
            });
            let editor_group = PreferencesGroup::new();
            editor_group.add(&vim_row);
            editor_group.add(&line_numbers_row);
            editor_group.add(&word_wrap_row);
            let editor_page = PreferencesPage::builder()
                .title("Editor")
                .icon_name("accessories-text-editor-symbolic")
                .build();
            editor_page.add(&editor_group);

            // Synchronization Page
            let sync_group = PreferencesGroup::new();
            sync_group.set_title("Synchronization");

            let sync_scroll_row = SwitchRow::builder()
                .title("Sync Scroll")
                .subtitle("Preview pane follows editor scroll")
                .active(load_pref(PREF_SYNC_SCROLL, DEFAULT_SYNC_SCROLL) == "true")
                .build();
            sync_scroll_row.connect_active_notify(|row| {
                save_pref(PREF_SYNC_SCROLL, if row.is_active() { "true" } else { "false" });
            });

            let dom_injection_row = SwitchRow::builder()
                .title("DOM Injection")
                .subtitle("Smoothly update preview without reloading")
                .active(load_pref(PREF_DOM_INJECTION, DEFAULT_DOM_INJECTION) == "true")
                .build();
            dom_injection_row.connect_active_notify(|row| {
                save_pref(PREF_DOM_INJECTION, if row.is_active() { "true" } else { "false" });
            });

            let debounce_adjustment = gtk4::Adjustment::new(
                load_pref(PREF_DEBOUNCE, DEFAULT_DEBOUNCE).parse::<f64>().unwrap_or(150.0),
                0.0, 5000.0, 10.0, 100.0, 0.0
            );
            let debounce_row = adw::SpinRow::builder()
                .title("Debounce Duration (ms)")
                .subtitle("Delay before updating preview (0 for live)")
                .adjustment(&debounce_adjustment)
                .build();
            debounce_row.connect_value_notify(|row| {
                save_pref(PREF_DEBOUNCE, &format!("{:.0}", row.value()));
            });

            sync_group.add(&sync_scroll_row);
            sync_group.add(&dom_injection_row);
            sync_group.add(&debounce_row);

            let sync_page = PreferencesPage::builder()
                .title("Synchronization")
                .icon_name("view-refresh-symbolic")
                .build();
            sync_page.add(&sync_group);

            // Interface Page
            let ui_group = PreferencesGroup::new();
            ui_group.set_title("Interface");

            let metadata_model = gio::ListStore::new::<StringObject>();
            metadata_model.append(&StringObject::new("Show Metadata"));
            metadata_model.append(&StringObject::new("Hide Metadata"));
            metadata_model.append(&StringObject::new("Ignore Metadata"));
            let metadata_expr = PropertyExpression::new(StringObject::static_type(), None::<&gtk4::Expression>, "string");
            let metadata_row = ComboRow::builder()
                .title("Metadata Handling")
                .subtitle("How to treat YAML headers")
                .model(&metadata_model)
                .expression(&metadata_expr)
                .build();
            let saved_metadata = load_pref(PREF_SHOW_METADATA, DEFAULT_SHOW_METADATA);
            metadata_row.set_selected(match saved_metadata.as_str() {
                "hide" => 1,
                "ignore" => 2,
                _ => 0,
            });
            metadata_row.connect_selected_notify({
                let refresh_preview = refresh_preview.clone();
                let sb = sb_pref.clone();
                move |row| {
                    save_pref(
                        PREF_SHOW_METADATA,
                        match row.selected() {
                            1 => "hide",
                            2 => "ignore",
                            _ => "show",
                        },
                    );
                    refresh_preview(&sb, true);
                }
            });

            let view_model = gio::ListStore::new::<StringObject>();
            view_model.append(&StringObject::new("Dual Pane"));
            view_model.append(&StringObject::new("Preview Only"));
            let view_expr = PropertyExpression::new(StringObject::static_type(), None::<&gtk4::Expression>, "string");
            let view_row = ComboRow::builder()
                .title("Default View Mode")
                .subtitle("Initial layout when opening a file")
                .model(&view_model)
                .expression(&view_expr)
                .build();
            let saved_view = load_pref(PREF_DEFAULT_VIEW, DEFAULT_VIEW);
            view_row.set_selected(match saved_view.as_str() {
                "preview-only" => 1,
                _ => 0,
            });
            view_row.connect_selected_notify({
                let refresh_preview = refresh_preview.clone();
                let sb = sb_pref.clone();
                move |row| {
                    save_pref(
                        PREF_DEFAULT_VIEW,
                        match row.selected() {
                            1 => "preview-only",
                            _ => "dual-pane",
                        },
                    );
                    refresh_preview(&sb, true);
                }
            });

            let readable_row = SwitchRow::builder()
                .title("Readable Line Length")
                .subtitle("Limit preview content width for better readability")
                .active(load_pref(PREF_READABLE_LINE, DEFAULT_READABLE_LINE) == "true")
                .build();

            let width_adjustment = gtk4::Adjustment::new(
                load_pref(PREF_MAX_WIDTH, DEFAULT_MAX_WIDTH).parse::<f64>().unwrap_or(1000.0),
                400.0, 3000.0, 50.0, 200.0, 0.0
            );
            let width_row = adw::SpinRow::builder()
                .title("Max Content Width (px)")
                .subtitle("Maximum width when Readable Line Length is enabled")
                .adjustment(&width_adjustment)
                .sensitive(readable_row.is_active())
                .build();

            readable_row.connect_active_notify({
                let refresh_preview = refresh_preview.clone();
                let sb = sb_pref.clone();
                let width_row = width_row.clone();
                move |row| {
                    let active = row.is_active();
                    save_pref(PREF_READABLE_LINE, if active { "true" } else { "false" });
                    width_row.set_sensitive(active);
                    refresh_preview(&sb, true);
                }
            });
            width_row.connect_value_notify({
                let refresh_preview = refresh_preview.clone();
                let sb = sb_pref.clone();
                move |row| {
                    save_pref(PREF_MAX_WIDTH, &format!("{:.0}", row.value()));
                    refresh_preview(&sb, true);
                }
            });

            ui_group.add(&metadata_row);
            ui_group.add(&view_row);
            ui_group.add(&readable_row);
            ui_group.add(&width_row);

            let ui_page = PreferencesPage::builder()
                .title("Interface")
                .icon_name("window-new-symbolic")
                .build();
            ui_page.add(&ui_group);

            // Shortcuts Page
            let shortcuts_group = PreferencesGroup::new();
            shortcuts_group.set_title("Keyboard Shortcuts");

            let create_shortcut_row = |title: &str, pref_key: &'static str, default: &str, app_ptr: Application, parent_win: &adw::ApplicationWindow| {
                let initial_subtitle = html_escape(&load_pref(pref_key, default));
                let row = adw::ActionRow::builder()
                    .title(title)
                    .subtitle(&initial_subtitle)
                    .use_markup(true)
                    .build();
                
                let edit_button = Button::builder()
                    .label("Edit")
                    .valign(gtk4::Align::Center)
                    .build();

                let app_ptr = app_ptr.clone();
                let parent_win = parent_win.clone();
                let row_clone = row.clone();
                let pref_key_static: &'static str = pref_key;
                let default_static = default.to_string();
                let title_owned = title.to_string();

                edit_button.connect_clicked(move |_| {
                    let grabber = adw::Window::builder()
                        .title("Grab Shortcut")
                        .modal(true)
                        .transient_for(&parent_win)
                        .default_width(300)
                        .default_height(150)
                        .build();

                    let content = Box::new(Orientation::Vertical, 10);
                    content.set_margin_top(20);
                    content.set_margin_bottom(20);
                    content.set_margin_start(20);
                    content.set_margin_end(20);
                    
                    let label = gtk4::Label::builder()
                        .label(&format!("Press keys for: {}", title_owned))
                        .use_markup(false)
                        .build();
                    let current_label = gtk4::Label::builder()
                        .label(&html_escape(&load_pref(pref_key_static, &default_static)))
                        .css_classes(vec!["title-2".to_string()])
                        .use_markup(true)
                        .build();
                    
                    let hint = gtk4::Label::builder()
                        .label("Press Enter to Save, Esc to Reset")
                        .use_markup(false)
                        .build();
                    hint.add_css_class("dim-label");

                    content.append(&label);
                    content.append(&current_label);
                    content.append(&hint);
                    grabber.set_content(Some(&content));

                    let recorded_accel = Rc::new(RefCell::new(load_pref(pref_key_static, &default_static)));

                    let key_ctrl = EventControllerKey::new();
                    let recorded_accel_inner = recorded_accel.clone();
                    let current_label_inner = current_label.clone();
                    let grabber_inner = grabber.clone();
                    let app_inner = app_ptr.clone();
                    let row_inner = row_clone.clone();
                    let default_inner = default_static.clone();

                    key_ctrl.connect_key_pressed(move |_, key, _, modifier| {
                        let key_name = key.name().unwrap_or_else(|| "unknown".into());

                        if key_name == "Return" {
                            let accel = recorded_accel_inner.borrow();
                            save_pref(pref_key_static, &accel);
                            row_inner.set_subtitle(&html_escape(&accel));
                            apply_accels(&app_inner);
                            grabber_inner.close();
                            return glib::Propagation::Stop;
                        }

                        if key_name == "Escape" {
                            save_pref(pref_key_static, &default_inner);
                            row_inner.set_subtitle(&html_escape(&default_inner));
                            apply_accels(&app_inner);
                            grabber_inner.close();
                            return glib::Propagation::Stop;
                        }

                        // Ignore pure modifier presses
                        if key_name.contains("Control") || key_name.contains("Shift") || 
                           key_name.contains("Alt") || key_name.contains("Super") || key_name.contains("Meta") {
                            return glib::Propagation::Stop;
                        }

                        let mut accel = String::new();
                        if modifier.contains(gtk4::gdk::ModifierType::CONTROL_MASK) { accel.push_str("<Control>"); }
                        if modifier.contains(gtk4::gdk::ModifierType::SHIFT_MASK) { accel.push_str("<Shift>"); }
                        if modifier.contains(gtk4::gdk::ModifierType::ALT_MASK) { accel.push_str("<Alt>"); }
                        if modifier.contains(gtk4::gdk::ModifierType::SUPER_MASK) || modifier.contains(gtk4::gdk::ModifierType::META_MASK) { 
                            accel.push_str("<Super>"); 
                        }
                        
                        accel.push_str(&key_name);
                        *recorded_accel_inner.borrow_mut() = accel.clone();
                        current_label_inner.set_text(&html_escape(&accel));

                        glib::Propagation::Stop
                    });

                    grabber.add_controller(key_ctrl);
                    grabber.present();
                });

                row.add_suffix(&edit_button);
                row
            };

            shortcuts_group.add(&create_shortcut_row("Open File", PREF_KEY_OPEN, DEFAULT_KEY_OPEN, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Save File", PREF_KEY_SAVE, DEFAULT_KEY_SAVE, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Save As", PREF_KEY_SAVE_AS, DEFAULT_KEY_SAVE_AS, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Export PDF", PREF_KEY_EXPORT_PDF, DEFAULT_KEY_EXPORT_PDF, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Open Preferences", PREF_KEY_PREFS, DEFAULT_KEY_PREFS, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Search &amp; Replace", PREF_KEY_SEARCH, DEFAULT_KEY_SEARCH, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Toggle Editor Pane", PREF_KEY_TOGGLE_EDITOR, DEFAULT_KEY_TOGGLE_EDITOR, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Cycle View Modes", PREF_KEY_CYCLE_VIEW, DEFAULT_KEY_CYCLE_VIEW, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Toggle Readable Width", PREF_KEY_TOGGLE_READABLE, DEFAULT_KEY_TOGGLE_READABLE, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Keyboard Shortcuts Dialog", PREF_KEY_SHORTCUTS, DEFAULT_KEY_SHORTCUTS, app.clone(), &w));
            shortcuts_group.add(&create_shortcut_row("Quit", PREF_KEY_QUIT, DEFAULT_KEY_QUIT, app.clone(), &w));

            let shortcuts_page = PreferencesPage::builder()
                .title("Shortcuts")
                .icon_name("preferences-desktop-keyboard-shortcuts-symbolic")
                .build();
            shortcuts_page.add(&shortcuts_group);

            let prefs = PreferencesDialog::builder()
                .title("Preferences")
                .build();
            prefs.add(&appearance_page);
            prefs.add(&editor_page);
            prefs.add(&sync_page);
            prefs.add(&ui_page);
            prefs.add(&shortcuts_page);
            prefs.present(Some(&w));
        });
    }
    app.add_action(&preferences_action);

    // Search
    let search_action = gio::SimpleAction::new("search", None);
    {
        let sb = search_bar.clone();
        search_action.connect_activate(move |_, _| {
            let is_visible = sb.property::<bool>("search-mode-enabled");
            sb.set_property("search-mode-enabled", !is_visible);
        });
    }
    app.add_action(&search_action);

    // About
    let about_action = gio::SimpleAction::new("about", None);
    {
        let w = window.clone();
        about_action.connect_activate(move |_, _| {
            let dlg = AboutDialog::builder()
                .application_name("MarkView")
                .version("1.1.0")
                .developer_name("Vaibhav Pratap Singh")
                .developers(vec!["Vaibhav Pratap Singh"])
                .copyright("© 2026")
                .website("https://github.com/v8v88v8v88/MarkView")
                .license_type(gtk4::License::Gpl30)
                .build();
            dlg.present(Some(&w));
        });
    }
    app.add_action(&about_action);

    // Keyboard Shortcuts
    let shortcuts_action = gio::SimpleAction::new("shortcuts", None);
    {
        let w = window.clone();
        shortcuts_action.connect_activate(move |_, _| {
            let file_section = ShortcutsSection::new(Some("File"));
            file_section.add(ShortcutsItem::from_action("Open", "app.open"));
            file_section.add(ShortcutsItem::from_action("Save", "app.save"));
            file_section.add(ShortcutsItem::from_action("Save As", "app.save-as"));
            file_section.add(ShortcutsItem::from_action("Export as PDF", "app.export-pdf"));
                        let app_section = ShortcutsSection::new(Some("Application"));
                        app_section.add(ShortcutsItem::from_action("Preferences", "app.preferences"));
                        app_section.add(ShortcutsItem::from_action("Keyboard Shortcuts", "app.shortcuts"));
                        app_section.add(ShortcutsItem::from_action("Quit", "app.quit"));
            let dlg = ShortcutsDialog::builder()
                .title("Keyboard Shortcuts")
                .build();
            dlg.add(file_section);
            dlg.add(app_section);
            dlg.present(Some(&w));
        });
    }
    app.add_action(&shortcuts_action);

    // Quit
    let quit_action = gio::SimpleAction::new("quit", None);
    {
        let w = window.clone();
        quit_action.connect_activate(move |_, _| w.close());
    }
    app.add_action(&quit_action);

    apply_accels(app);

    window.present();

    if let Some(file) = initial_file {
        if let Some(path) = file.path() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    source_buffer.set_text(&content);
                    if let Some(name) = path.file_name() {
                        window.set_title(Some(&format!("{} — MarkView", name.to_string_lossy())));
                    }
                }
                Err(e) => eprintln!("Failed to read file: {e}"),
            }
        }
    }
}

fn main() {
    let app = Application::builder()
        .application_id("com.v8v88v8v88.MarkView")
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_activate(|app| {
        build_ui(app, None);
    });

    app.connect_open(|app, files, _| {
        for file in files {
            build_ui(app, Some(file.clone()));
            break;
        }
    });

    app.run();
}