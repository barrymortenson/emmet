// SPDX-License-Identifier: AGPL-3.0-or-later
// The title row (docs/design/frozen's titlebar): the project chip (the ant and the project's name) and its menu (New,
// Open, Save version, Versions, Export, About), with Save version and the versions list as popovers anchored to it;
// the view switch, centered; and on Windows the menu bar (File, Edit, View, Agent, Help) and the caption buttons. The
// row is the window's drag region.
import { getCurrentWindow } from "@tauri-apps/api/window";
import { DropdownMenu, Menubar, Popover, ToggleGroup } from "radix-ui";
import { useEffect, useRef, useState } from "react";
import ant from "./assets/brand/mark-ant.svg";
import type { VersionView } from "./bindings";
import { type Action, keysFor, windows } from "./keys";
import { Badge, Button, IconButton, Icon, MenuItems, type MenuEntry, when } from "./ui";

export type Pop = "save" | "versions" | null;
type View = "iso" | "front" | "top";

/** Windows' menus: the same as macOS's menu bar (keyboard.md), in sentence case. */
function windowsMenus(run: (a: Action) => void, can: (a: Action) => boolean): [string, MenuEntry[]][] {
  const item = (id: Action, text: string): MenuEntry => ({ id, text, keys: keysFor(id), disabled: !can(id), onSelect: () => run(id) });
  return [
    ["File", [item("new", "New"), item("open", "Open…"), "-", item("save", "Save version"), item("versions", "Versions…"), item("export", "Export…"), "-", item("settings", "Settings…")]],
    ["Edit", [item("undo-knob", "Undo"), item("redo-knob", "Redo"), "-", item("undo-turn", "Undo the agent's last turn")]],
    [
      "View",
      [
        item("view:iso", "Iso"),
        item("view:front", "Front"),
        item("view:top", "Top"),
        item("view:back", "Back"),
        item("view:left", "Left"),
        item("view:right", "Right"),
        item("frame", "Frame"),
        "-",
        item("tab:options", "Options"),
        item("tab:checks", "Checks"),
        item("tab:parts", "Parts"),
        item("tab:steps", "Steps"),
        "-",
        item("step:prev", "Previous step"),
        item("step:next", "Next step"),
        "-",
        item("history", "Show chat history"),
      ],
    ],
    ["Agent", [item("agent:message", "Go to message box"), item("agent:send", "Send"), item("agent:stop", "Stop"), "-", item("agent:fresh", "Start fresh"), item("agent:switch", "Switch agent"), item("agent:model", "Model and effort…"), item("agent:usage", "Plan usage…")]],
    ["Help", [item("help:guide", "Emmet guide"), item("shortcuts", "Keyboard shortcuts"), item("diagnostics", "Copy diagnostics"), item("help:report", "Report a problem…"), "-", item("updates", "Check for updates…"), item("cli", "Install command line tool…"), item("about", "About Emmet Builder")]],
  ];
}

function SavePopover({ onSave, onCancel }: { onSave: (name: string) => Promise<void>; onCancel: () => void }) {
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const save = async () => {
    if (!name.trim() || busy) return;
    setBusy(true);
    await onSave(name.trim());
    setBusy(false);
  };
  return (
    <>
      <span className="popover-title">Save version</span>
      <input
        className="text-field"
        data-c="text-field"
        type="text"
        aria-label="Version name"
        placeholder="A few words, like Steeper roof"
        value={name}
        autoFocus
        id="version-name"
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && void save()}
      />
      <div className="popover-actions">
        <Button label="Cancel" variant="quiet" size="sm" onClick={onCancel} />
        <Button label="Save" variant="primary" size="sm" icon="save" id="version-save" disabled={!name.trim()} busy={busy} onClick={() => void save()} />
      </div>
    </>
  );
}

function VersionsPopover({ versions, onSave, onView }: { versions: VersionView[] | null; onSave: () => void; onView: (v: VersionView) => void }) {
  return (
    <>
      <span className="popover-title">Versions</span>
      <div className="popover-head">
        <Badge text={versions ? `${versions.length} version${versions.length === 1 ? "" : "s"}` : "…"} />
        <Button label="Save version" size="sm" icon="save" onClick={onSave} />
      </div>
      <div className="versions-list" id="versions-list">
        {versions?.length === 0 && <p className="small">Save one any time and keep going: you can always come back to it.</p>}
        {versions?.map((v) => (
          <button key={v.n} type="button" className="version-row" data-c="version-row" data-auto={v.auto} data-n={v.n} disabled={!v.viewable} onClick={() => onView(v)}>
            {v.thumbnail ? <img src={v.thumbnail} alt="" /> : <span className="version-thumb-empty" aria-hidden="true" />}
            <span className="version-row-text">
              <b>{v.name}</b>
              <span className="small">
                {when(v.time)}
                {v.parts != null && ` · ${v.parts} parts`}
                {v.auto && " · saved by Emmet"}
              </span>
            </span>
          </button>
        ))}
      </div>
    </>
  );
}

export function Titlebar({
  name,
  hasProject,
  view,
  onView,
  run,
  can,
  pop,
  onPop,
  versions,
  onSaveVersion,
  onOpenVersion,
}: {
  name: string;
  hasProject: boolean;
  view: View | null;
  onView: (v: View) => void;
  run: (a: Action) => void;
  can: (a: Action) => boolean;
  pop: Pop;
  onPop: (p: Pop) => void;
  versions: VersionView[] | null;
  onSaveVersion: (name: string) => Promise<void>;
  onOpenVersion: (v: VersionView) => void;
}) {
  const [menu, setMenu] = useState(false);
  /** A menu item that opens a popover on the chip was chosen: focus mustn't go back to the chip, which would close it. */
  const popping = useRef(false);
  const [maximized, setMaximized] = useState(false);
  useEffect(() => {
    if (!windows) return;
    const w = getCurrentWindow();
    void w.isMaximized().then(setMaximized);
    const off = w.onResized(() => void w.isMaximized().then(setMaximized));
    return () => void off.then((f) => f());
  }, []);
  const entry = (id: Action, text: string, icon: Parameters<typeof Icon>[0]["name"]): MenuEntry => ({
    id,
    text,
    icon,
    keys: keysFor(id),
    disabled: !can(id),
    onSelect: () => {
      popping.current = id === "save" || id === "versions";
      run(id);
    },
  });
  return (
    <header className="titlebar" data-c="titlebar" data-tauri-drag-region>
      <div className="titlebar-start" data-tauri-drag-region>
        <Popover.Root open={pop != null} onOpenChange={(o) => !o && onPop(null)}>
          <DropdownMenu.Root open={menu} onOpenChange={setMenu}>
            <Popover.Anchor asChild>
              <DropdownMenu.Trigger className="project-chip" id="project-chip">
                <img src={ant} alt="" />
                {name}
                <Icon name="chevron-down" size="sm" />
              </DropdownMenu.Trigger>
            </Popover.Anchor>
            <DropdownMenu.Portal>
              <DropdownMenu.Content
                className="menu"
                data-c="menu"
                aria-label="Project"
                align="start"
                sideOffset={6}
                id="project-menu"
                onCloseAutoFocus={(e) => {
                  if (popping.current) e.preventDefault();
                  popping.current = false;
                }}
              >
                <MenuItems
                  items={[
                    entry("new", "New", "plus"),
                    entry("open", "Open…", "folder-open"),
                    "-",
                    entry("save", "Save version", "save"),
                    entry("versions", "Versions…", "rotate-ccw-clock"),
                    "-",
                    entry("export", "Export…", "download"),
                    "-",
                    entry("about", "About Emmet Builder", "info"),
                  ]}
                />
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
          <Popover.Portal>
            <Popover.Content className="popover" data-c="popover" aria-label={pop === "save" ? "Save version" : "Versions"} align="start" sideOffset={6} id={`pop-${pop}`} onOpenAutoFocus={(e) => pop === "versions" && e.preventDefault()}>
              {pop === "save" && <SavePopover onSave={onSaveVersion} onCancel={() => onPop(null)} />}
              {pop === "versions" && <VersionsPopover versions={versions} onSave={() => onPop("save")} onView={onOpenVersion} />}
            </Popover.Content>
          </Popover.Portal>
        </Popover.Root>
        {windows && (
          <Menubar.Root className="menubar" data-c="menubar" aria-label="Menu">
            {windowsMenus(run, can).map(([title, items]) => (
              <Menubar.Menu key={title}>
                <Menubar.Trigger className="menubar-trigger">{title}</Menubar.Trigger>
                <Menubar.Portal>
                  <Menubar.Content className="menu" data-c="menu" align="start" sideOffset={4}>
                    <MenuItems items={items} />
                  </Menubar.Content>
                </Menubar.Portal>
              </Menubar.Menu>
            ))}
          </Menubar.Root>
        )}
      </div>
      <div className="view-switch" data-c="view-switch" role="group" aria-label="View">
        <ToggleGroup.Root type="single" className="view-switch-views" aria-label="Views" value={view ?? ""} onValueChange={(v) => v && onView(v as View)} disabled={!hasProject}>
          {(["iso", "front", "top"] as const).map((v, i) => (
            <ToggleGroup.Item key={v} value={v} className="view-switch-item" title={`${v[0]!.toUpperCase()}${v.slice(1)} (${i + 1})`} id={`view-${v}`}>
              {v[0]!.toUpperCase() + v.slice(1)}
            </ToggleGroup.Item>
          ))}
        </ToggleGroup.Root>
        <IconButton icon="scan" label="Frame the model (F)" size="sm" onClick={() => run("frame")} disabled={!hasProject} id="frame" />
        <IconButton icon="camera" label="Screenshot" size="sm" disabled id="screenshot" />
      </div>
      <span />
      {windows && (
        <div className="caption-buttons" data-c="caption-buttons">
          <button type="button" className="caption-button" data-kind="minimize" aria-label="Minimize" onClick={() => void getCurrentWindow().minimize()}>
            <Icon name="minus" size="sm" />
          </button>
          <button type="button" className="caption-button" data-kind="maximize" aria-label={maximized ? "Restore" : "Maximize"} onClick={() => void getCurrentWindow().toggleMaximize()}>
            <Icon name="square" size="sm" />
          </button>
          <button type="button" className="caption-button" data-kind="close" aria-label="Close" onClick={() => void getCurrentWindow().close()}>
            <Icon name="x" size="sm" />
          </button>
        </div>
      )}
    </header>
  );
}
