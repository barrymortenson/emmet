// SPDX-License-Identifier: AGPL-3.0-or-later
//! macOS's menu bar (docs/design/keyboard.md, session 4A). Each item's click goes to the window as a [`MenuAction`]
//! with the same id the window's keys and its Windows menu bar use, so every action has one handler. Keys that type a
//! character (1–6, F, [ and ]) and Esc aren't menu accelerators: a menu would take them from a text field, so the
//! window handles them. Open Recent's items open their project here, so the window never names a path. Windows draws
//! the same menus in its title row (Menubar.tsx).

use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, Submenu, SubmenuBuilder};
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::app::AppState;

/// A menu item the window acts on: "new", "open", "save", "view:iso", "tab:checks", …
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type, tauri_specta::Event)]
pub struct MenuAction {
    pub id: String,
}

/// The items 4A can't act on yet (the chat's own, 4B; the guide, 6B), shown but not enabled.
const LATER: [&str; 3] = ["agent:switch", "agent:model", "help:guide"];

fn item<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
    text: &str,
    key: Option<&str>,
) -> tauri::Result<tauri::menu::MenuItem<R>> {
    let mut b = MenuItemBuilder::with_id(id, text).enabled(!LATER.contains(&id));
    if let Some(k) = key {
        b = b.accelerator(k);
    }
    b.build(app)
}

fn recent<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    let list = crate::recent::list(&app.state::<AppState>().data);
    let mut b = SubmenuBuilder::new(app, "Open Recent");
    if list.is_empty() {
        b = b.item(
            &MenuItemBuilder::with_id("recent:none", "No Recent Projects")
                .enabled(false)
                .build(app)?,
        );
    }
    for (i, r) in list.iter().enumerate() {
        b = b.item(&MenuItemBuilder::with_id(format!("recent:{i}"), &r.name).build(app)?);
    }
    b.build()
}

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let sep = || PredefinedMenuItem::separator(app);
    let emmet = SubmenuBuilder::new(app, "Emmet Builder")
        .item(&item(app, "about", "About Emmet Builder", None)?)
        .item(&sep()?)
        .item(&item(app, "settings", "Settings…", Some("CmdOrCtrl+,"))?)
        .item(&item(app, "updates", "Check for Updates…", None)?)
        .item(&item(app, "cli", "Install Command Line Tool…", None)?)
        .item(&sep()?)
        .item(&PredefinedMenuItem::hide(app, None)?)
        .item(&PredefinedMenuItem::hide_others(app, None)?)
        .item(&PredefinedMenuItem::show_all(app, None)?)
        .item(&sep()?)
        .item(&PredefinedMenuItem::quit(app, None)?)
        .build()?;
    let file = SubmenuBuilder::new(app, "File")
        .item(&item(app, "new", "New", Some("CmdOrCtrl+N"))?)
        .item(&item(app, "open", "Open…", Some("CmdOrCtrl+O"))?)
        .item(&recent(app)?)
        .item(&sep()?)
        .item(&item(app, "save", "Save Version", Some("CmdOrCtrl+S"))?)
        .item(&item(app, "versions", "Versions…", None)?)
        .item(&item(app, "export", "Export…", Some("CmdOrCtrl+E"))?)
        .item(&sep()?)
        .item(&PredefinedMenuItem::close_window(app, None)?)
        .build()?;
    let edit = SubmenuBuilder::new(app, "Edit")
        .item(&PredefinedMenuItem::undo(app, None)?)
        .item(&PredefinedMenuItem::redo(app, None)?)
        .item(&sep()?)
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&PredefinedMenuItem::paste(app, None)?)
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .item(&sep()?)
        .item(&item(
            app,
            "undo-turn",
            "Undo the Agent's Last Turn",
            Some("Alt+CmdOrCtrl+Z"),
        )?)
        .build()?;
    let mut view = SubmenuBuilder::new(app, "View");
    for (id, text) in [
        ("iso", "Iso"),
        ("front", "Front"),
        ("top", "Top"),
        ("back", "Back"),
        ("left", "Left"),
        ("right", "Right"),
    ] {
        view = view.item(&item(app, &format!("view:{id}"), text, None)?);
    }
    view = view.item(&item(app, "frame", "Frame", None)?).item(&sep()?);
    for (i, (id, text)) in [
        ("options", "Options"),
        ("checks", "Checks"),
        ("parts", "Parts"),
        ("steps", "Steps"),
    ]
    .iter()
    .enumerate()
    {
        view = view.item(&item(
            app,
            &format!("tab:{id}"),
            text,
            Some(&format!("CmdOrCtrl+{}", i + 1)),
        )?);
    }
    let view = view
        .item(&sep()?)
        .item(&item(app, "step:prev", "Previous Step", None)?)
        .item(&item(app, "step:next", "Next Step", None)?)
        .item(&sep()?)
        .item(&item(app, "history", "Show Chat History", None)?)
        .item(&PredefinedMenuItem::fullscreen(app, None)?)
        .build()?;
    let agent = SubmenuBuilder::new(app, "Agent")
        .item(&item(app, "agent:message", "Go to Message Box", Some("CmdOrCtrl+L"))?)
        .item(&item(app, "agent:send", "Send", None)?)
        .item(&item(app, "agent:stop", "Stop", None)?)
        .item(&sep()?)
        .item(&item(app, "agent:fresh", "Start Fresh", None)?)
        .item(&item(app, "agent:switch", "Switch Agent", None)?)
        .item(&item(app, "agent:model", "Model and Effort…", None)?)
        .item(&item(app, "agent:usage", "Plan Usage…", None)?)
        .build()?;
    let window = SubmenuBuilder::new(app, "Window")
        .item(&PredefinedMenuItem::minimize(app, None)?)
        .item(&PredefinedMenuItem::maximize(app, Some("Zoom"))?)
        .item(&sep()?)
        .item(&PredefinedMenuItem::bring_all_to_front(app, None)?)
        .build()?;
    let help = SubmenuBuilder::new(app, "Help")
        .item(&item(app, "help:guide", "Emmet Guide", None)?)
        .item(&item(app, "shortcuts", "Keyboard Shortcuts", Some("CmdOrCtrl+/"))?)
        .item(&sep()?)
        .item(&item(app, "diagnostics", "Copy Diagnostics", None)?)
        .item(&item(app, "help:report", "Report a Problem…", None)?)
        .build()?;
    MenuBuilder::new(app)
        .items(&[&emmet, &file, &edit, &view, &agent, &window, &help])
        .build()
}

/// Puts the menu bar up (macOS only) and sends its clicks to the window.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }
    app.set_menu(build(app)?)?;
    app.on_menu_event(|app, event| {
        let id = event.id().as_ref().to_string();
        if let Some(i) = id.strip_prefix("recent:").and_then(|i| i.parse::<usize>().ok()) {
            let list = crate::recent::list(&app.state::<AppState>().data);
            if let Some(r) = list.get(i) {
                let path = std::path::PathBuf::from(&r.path);
                if let Err(e) = crate::app::open(app, &path) {
                    app.state::<AppState>()
                        .notify(format!("Couldn't open {}: {} {}", r.name, e.message, e.hint));
                }
            }
            return;
        }
        let _ = MenuAction { id }.emit(app);
    });
    Ok(())
}

/// Open Recent changes when a project opens: the menu is built again.
pub fn refresh(app: &AppHandle) {
    if cfg!(target_os = "macos")
        && let Ok(menu) = build(app)
    {
        let _ = app.set_menu(menu);
    }
}
