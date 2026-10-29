// SPDX-License-Identifier: AGPL-3.0-or-later
// The options panel (plan section 14, docs/design/frozen): the model's state in a word with its icon (No problems,
// 1 problem, Building, Build failed, Read-only, Empty) and Export, then the four tabs (Radix Tabs; ⌘1–⌘4 jump to one):
// Options, Checks, Parts, and Steps.
import { DropdownMenu, Tabs } from "radix-ui";
import { useEffect } from "react";
import type { AllowedView, BomLine, BuildSettings, Finding, Problem, ProjectView, Scene } from "./bindings";
import { Checks, checksCount } from "./Checks";
import { keysFor } from "./keys";
import { BuildSettingsSection, Knobs } from "./Options";
import { Parts } from "./Parts";
import type { ExportFormat } from "./bindings";
import { Steps } from "./Steps";
import { Badge, Button, Icon, MenuItems, Spinner } from "./ui";

export type Tab = "options" | "checks" | "parts" | "steps";
export const TABS: [Tab, string][] = [
  ["options", "Options"],
  ["checks", "Checks"],
  ["parts", "Parts"],
  ["steps", "Steps"],
];

type State = "empty" | "building" | "failed" | "readonly" | "findings" | "clean";

export function modelState(project: ProjectView | null, scene: Scene | null, running: boolean, error: Problem | null): State {
  if (project && !project.trusted) return "readonly";
  if (running) return "building";
  if (error) return "failed";
  if (!scene || scene.parts <= 1) return "empty";
  return checksCount(scene.checks) ? "findings" : "clean";
}

const HEAD: Record<State, { tone: string; icon?: "octagon-alert" | "lock" | "triangle-alert" | "info" | "circle-check" }> = {
  failed: { tone: "bad", icon: "octagon-alert" },
  findings: { tone: "warn", icon: "triangle-alert" },
  building: { tone: "busy" },
  readonly: { tone: "warn", icon: "lock" },
  empty: { tone: "muted", icon: "info" },
  clean: { tone: "ok", icon: "circle-check" },
};

function stateWords(s: State, n: number): string {
  return { empty: "Empty", building: "Building", failed: "Build failed", readonly: "Read-only", clean: "No problems", findings: `${n} problem${n === 1 ? "" : "s"}` }[s];
}

export type PanelProps = {
  project: ProjectView;
  scene: Scene | null;
  running: boolean;
  error: Problem | null;
  tab: Tab;
  onTab: (t: Tab) => void;
  selected: string | null;
  step: number | null;
  sub: string | null;
  exportOpen: boolean;
  onExportOpen: (open: boolean) => void;
  onExport: (f: ExportFormat) => void;
  onPickFinding: (f: Finding) => void;
  onAsk: (f: Finding) => void;
  onAllow: (f: Finding) => void;
  onUnallow: (a: AllowedView) => void;
  onPickPart: (l: BomLine) => void;
  onStep: (n: number) => void;
  onSub: (file: string | null) => void;
  onSaveBuild: (s: BuildSettings) => Promise<boolean>;
  onRebuild: (before: BuildSettings, after: BuildSettings) => void;
  onKnobWrite: () => void;
};

export function Panel(p: PanelProps) {
  const state = modelState(p.project, p.scene, p.running, p.error);
  const n = checksCount(p.scene?.checks);
  const head = HEAD[state];
  const counts: Partial<Record<Tab, number>> = {
    checks: n,
    parts: p.scene?.bom?.pieces ?? 0,
    steps: p.scene?.tree?.steps.length ?? 0,
  };
  const readonly = !p.project.trusted;
  const fresh = state === "empty";
  const exportable = state !== "empty" && (p.scene?.parts ?? 0) > 0;
  // ⌘E with no Export button leaves nothing to open later, when the button comes.
  const { exportOpen, onExportOpen } = p;
  useEffect(() => {
    if (exportOpen && !exportable) onExportOpen(false);
  }, [exportOpen, exportable, onExportOpen]);
  return (
    <aside className="panel" data-c="panel" aria-label="Options" id="panel" tabIndex={-1}>
      <div className="panel-head" data-tone={head.tone} id="model-state">
        {head.tone === "busy" ? <Spinner label="Building" /> : <Icon name={head.icon!} />}
        <span className="panel-head-text">{stateWords(state, n)}</span>
        {exportable && (
          <DropdownMenu.Root open={p.exportOpen} onOpenChange={p.onExportOpen}>
            <DropdownMenu.Trigger asChild>
              <Button label="Export" size="sm" icon="download" title={`Export (${keysFor("export")})`} id="export-button" />
            </DropdownMenu.Trigger>
            <DropdownMenu.Portal>
              <DropdownMenu.Content className="menu" data-c="menu" aria-label="Export" align="end" sideOffset={6}>
                <MenuItems
                  items={[
                    { label: "Parts list" },
                    { id: "csv", icon: "sheet", text: "CSV", sub: "Every part and color, with counts", onSelect: () => p.onExport("csv") },
                    { id: "rebrickable", icon: "list", text: "Rebrickable", sub: "A parts list to import", onSelect: () => p.onExport("rebrickable") },
                    { id: "bricklink", icon: "list", text: "BrickLink XML", sub: "A wanted list", onSelect: () => p.onExport("bricklink-xml") },
                    { label: "Model" },
                    { id: "mpd", icon: "box", text: "LDraw .mpd", sub: "Opens in Studio, LDCad, and LeoCAD", onSelect: () => p.onExport("mpd") },
                    { label: "Instructions" },
                    // The PDF is 4E's.
                    { id: "pdf", icon: "file-text", text: "PDF", sub: "Step pages, A4 or Letter", disabled: true, onSelect: () => {} },
                    { id: "lpub3d", icon: "file-text", text: "LPub3D-ready file", sub: "For your own layout", onSelect: () => p.onExport("lpub3d") },
                  ]}
                />
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
        )}
      </div>
      <Tabs.Root value={p.tab} onValueChange={(t) => p.onTab(t as Tab)} className="panel-tabs">
        <Tabs.List className="tabs" data-c="tabs" aria-label="Options panel">
          {TABS.map(([id, label]) => (
            <Tabs.Trigger key={id} value={id} className="tab" id={`tab-${id}`} data-tab={id}>
              {label}
              {counts[id] ? <Badge text={String(counts[id])} tone={id === "checks" ? "warn" : "count"} /> : null}
            </Tabs.Trigger>
          ))}
        </Tabs.List>
        <Tabs.Content value="options" className="panel-body" id="options">
          <BuildSettingsSection saved={p.project.build} fresh={fresh} disabled={readonly} onSave={p.onSaveBuild} onRebuild={p.onRebuild} />
          {!fresh && !p.project.imported && (
            <section className="panel-section" aria-labelledby="tweaks-title">
              <div className="panel-section-head" id="tweaks-title">
                Tweaks
              </div>
              {readonly ? (
                <p className="muted row" id="tweaks-locked">
                  <Icon name="lock" size="sm" />
                  Trust the project to use these
                </p>
              ) : (
                <Knobs key={p.project.root} lastGeneration={p.scene?.generation ?? -1} onWrite={p.onKnobWrite} />
              )}
            </section>
          )}
        </Tabs.Content>
        <Tabs.Content value="checks" className="panel-body" id="checks">
          <Checks checks={p.scene?.checks} selected={p.selected?.startsWith("finding:") ? p.selected.slice(8) : null} readonly={readonly} onPick={p.onPickFinding} onAsk={p.onAsk} onAllow={p.onAllow} onUnallow={p.onUnallow} />
        </Tabs.Content>
        <Tabs.Content value="parts" className="panel-body" id="parts">
          <Parts bom={p.scene?.bom} selected={p.selected?.startsWith("part:") ? p.selected.slice(5) : null} onPick={p.onPickPart} />
        </Tabs.Content>
        <Tabs.Content value="steps" className="panel-body" id="steps">
          <Steps tree={p.scene?.tree} step={p.step} sub={p.sub} onStep={p.onStep} onSub={p.onSub} />
        </Tabs.Content>
      </Tabs.Root>
    </aside>
  );
}
