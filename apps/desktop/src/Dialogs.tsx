// SPDX-License-Identifier: AGPL-3.0-or-later
// The window's dialogs (Radix Dialog, docs/design/frozen): Settings (the agent, the projects folder, the look, Emmet's
// updates, the command line tool, and diagnostics; 1E's Codex sign-in and API key inside), About Emmet Builder (the
// versions, what Emmet connects to, the trademark line, and Licenses: the attributions, D-44, and the provider
// notice, D-37), and the keyboard shortcuts.
import { Dialog, Select, ToggleGroup } from "radix-ui";
import { type ReactNode, useEffect, useState } from "react";
import iconMac from "./assets/brand/app-icon-macos.svg";
import iconWin from "./assets/brand/app-icon-windows.svg";
import antDark from "./assets/brand/wordmark-emmet-builder-ant-dark.svg";
import antLight from "./assets/brand/wordmark-emmet-builder-ant-light.svg";
import { type AppInfo, type CliStatus, commands, type Prefs, type PrefsView } from "./bindings";
import { CodexAccounts } from "./ChatGpt";
import { KEYS, keyName, windows } from "./keys";
import { REBRICKABLE_CREDIT } from "./Parts";
import { Button, Icon } from "./ui";

function Modal({ title, id, className, onClose, children }: { title: string; id: string; className?: string; onClose: () => void; children: ReactNode }) {
  return (
    <Dialog.Root open onOpenChange={(o) => !o && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" data-c="dialog">
          <Dialog.Content className={`dialog${className ? ` ${className}` : ""}`} id={id} aria-describedby={undefined}>
            <Dialog.Title className={className === "about" ? "visually-hidden" : "dialog-title"}>{title}</Dialog.Title>
            {children}
          </Dialog.Content>
        </Dialog.Overlay>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

const Row = ({ label, children, id }: { label: string; children: ReactNode; id?: string }) => (
  <div className="setting" data-c="setting" data-inline id={id}>
    <span className="setting-name">{label}</span>
    {children}
  </div>
);

function Segmented<T extends string>({ label, value, items, onChange, id }: { label: string; value: T; items: [T, string][]; onChange: (v: T) => void; id?: string }) {
  return (
    <ToggleGroup.Root type="single" className="segmented" data-c="segmented" aria-label={label} value={value} onValueChange={(v) => v && onChange(v as T)} id={id}>
      {items.map(([v, text]) => (
        <ToggleGroup.Item key={v} value={v} className="segmented-item" data-value={v}>
          {text}
        </ToggleGroup.Item>
      ))}
    </ToggleGroup.Root>
  );
}

function Choose({ label, value, items, onChange, id }: { label: string; value: string; items: [string, string][]; onChange: (v: string) => void; id?: string }) {
  return (
    <Select.Root value={value} onValueChange={onChange}>
      <span className="select" data-c="select">
        <span className="visually-hidden">{label}</span>
        <Select.Trigger className="select-trigger" aria-label={label} id={id}>
          <Select.Value />
          <Icon name="chevron-down" size="sm" />
        </Select.Trigger>
      </span>
      <Select.Portal>
        <Select.Content className="menu" data-c="menu" position="popper" sideOffset={4}>
          <Select.Viewport>
            {items.map(([v, text]) => (
              <Select.Item key={v} value={v} className="menu-item">
                <Select.ItemText>{text}</Select.ItemText>
              </Select.Item>
            ))}
          </Select.Viewport>
        </Select.Content>
      </Select.Portal>
    </Select.Root>
  );
}

const LAST = "last";
const DEFAULT = "default";

export function Settings({ info, prefs, onPrefs, onClose, onAgentsChanged, onCopied }: { info: AppInfo | null; prefs: PrefsView | null; onPrefs: (p: PrefsView) => void; onClose: () => void; onAgentsChanged: () => void; onCopied: (text: string) => void }) {
  const [cli, setCli] = useState<CliStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [key, setKey] = useState("");
  const [keySaved, setKeySaved] = useState<boolean | null>(null);
  const [installing, setInstalling] = useState(false);
  useEffect(() => {
    void commands.cliStatus().then(setCli);
    void commands.openaiKeySaved().then(setKeySaved);
  }, []);
  if (!prefs) return null;
  const say = (r: { status: "error"; error: { message: string; hint: string } }) => setError(`${r.error.message} ${r.error.hint}`.trim());
  const set = async (change: Partial<Prefs>) => {
    const r = await commands.setPrefs({ ...prefs.prefs, ...change });
    if (r.status === "ok") onPrefs(r.data);
    else say(r);
  };
  const setKeyTo = async (value: string | null) => {
    const r = await commands.openaiKeySet(value);
    if (r.status === "ok") {
      setKeySaved(r.data);
      setKey("");
      setError(null);
      onAgentsChanged();
    } else say(r);
  };
  const folder = windows ? prefs.projects_folder.replaceAll("/", "\\") : prefs.projects_folder;
  return (
    <Modal title="Settings" id="settings" className="settings-dialog" onClose={onClose}>
      <section className="panel-section">
        <div className="panel-section-head">Agent</div>
        <Row label="Start with">
          <Choose
            label="Start with"
            id="start-with"
            value={prefs.prefs.start_with ?? LAST}
            items={[
              [LAST, "The last one used"],
              ["claude", "Claude"],
              ["codex", "Codex"],
            ]}
            onChange={(v) => void set({ start_with: v === LAST ? null : v })}
          />
        </Row>
        <Row label="Claude's model">
          <Choose
            label="Claude's model"
            id="claude-model"
            value={prefs.prefs.claude_model ?? DEFAULT}
            items={[
              [DEFAULT, "Claude Code's default"],
              ["opus", "Opus"],
              ["sonnet", "Sonnet"],
              ["haiku", "Haiku"],
            ]}
            onChange={(v) => void set({ claude_model: v === DEFAULT ? null : v })}
          />
        </Row>
        <div className="settings-codex" id="codex-billing-section">
          <span className="setting-name">Codex runs on</span>
          <CodexAccounts onChanged={onAgentsChanged} />
          <p className="small" id="openai-key-status">
            {keySaved == null ? "" : keySaved ? "An OpenAI API key is saved." : "No OpenAI API key yet."}
          </p>
          <form
            className="row"
            onSubmit={(e) => {
              e.preventDefault();
              if (key.trim()) void setKeyTo(key);
            }}
          >
            <input className="text-field grow" data-c="text-field" id="openai-key" type="password" autoComplete="off" spellCheck={false} aria-label="OpenAI API key" placeholder="OpenAI API key" value={key} onChange={(e) => setKey(e.target.value)} />
            <button type="submit" className="button" data-c="button" data-size="sm" disabled={!key.trim()}>
              <span>Save</span>
            </button>
            {keySaved && <Button label="Remove" size="sm" variant="quiet" onClick={() => void setKeyTo(null)} />}
          </form>
          <p className="small">Claude Code uses the sign-in you gave it. Emmet never changes it.</p>
        </div>
      </section>
      <section className="panel-section">
        <div className="panel-section-head">Projects</div>
        <Row label={`New projects go in ${folder}`} id="projects-folder">
          <Button
            label="Change…"
            size="sm"
            id="projects-folder-change"
            onClick={async () => {
              const r = await commands.pickProjectsFolder();
              if (r.status !== "ok") say(r);
              else if (r.data) onPrefs(r.data);
            }}
          />
        </Row>
      </section>
      <section className="panel-section">
        <div className="panel-section-head">Look</div>
        <Row label="Appearance">
          <Segmented<string> label="Appearance" id="appearance" value={prefs.prefs.appearance} items={[["system", "System"], ["light", "Light"], ["dark", "Dark"]]} onChange={(v) => void set({ appearance: v })} />
        </Row>
        <Row label="Color names">
          <Segmented<string>
            label="Color names"
            id="color-names"
            value={prefs.color_names}
            items={[["ldraw", "LDraw"], ["rebrickable", "Rebrickable"], ["bricklink", "BrickLink"]]}
            onChange={async (v) => {
              const r = await commands.setColorNames(v);
              if (r.status === "ok") onPrefs({ ...prefs, color_names: v });
              else say(r);
            }}
          />
        </Row>
        <Row label="Viewer quality">
          <Segmented<string> label="Viewer quality" id="viewer-quality" value={prefs.prefs.viewer_quality} items={[["fast", "Fast"], ["balanced", "Balanced"], ["best", "Best"]]} onChange={(v) => void set({ viewer_quality: v })} />
        </Row>
      </section>
      <section className="panel-section">
        <div className="panel-section-head">Emmet</div>
        <Row label="Updates">
          <span className="small" id="updates">
            {__EMMET_VERSION__} · Update checks start with the first release
          </span>
        </Row>
        <Row label="Parts data" id="parts-data">
          <span className="small" id="parts-data-status">
            {info?.parts_data ? `From ${info.parts_data}` : "None yet: availability shows as unknown"}
          </span>
        </Row>
        <Row label="Command line tool">
          {cli?.installed ? (
            <span className="small" id="cli-status">
              Installed in {cli.place}
            </span>
          ) : (
            <Button
              label="Install"
              size="sm"
              icon="terminal"
              id="cli-install"
              busy={installing}
              title={`Puts emmet in ${cli?.place ?? "your PATH"}${windows ? "" : "; macOS asks for your password"}`}
              onClick={async () => {
                setInstalling(true);
                const r = await commands.installCli();
                setInstalling(false);
                if (r.status !== "ok") say(r);
                else if (r.data) setCli(r.data);
              }}
            />
          )}
        </Row>
        <Row label="Diagnostics">
          <Button
            label="Copy diagnostics"
            size="sm"
            icon="copy"
            id="copy-diagnostics"
            onClick={async () => {
              const text = await commands.diagnostics();
              await navigator.clipboard.writeText(text).catch(() => {});
              onCopied(text);
            }}
          />
        </Row>
        {info?.parts_data && <p className="small">{REBRICKABLE_CREDIT}</p>}
      </section>
      {error && (
        <p className="notice" role="alert">
          {error}
        </p>
      )}
      <div className="dialog-actions">
        <Dialog.Close asChild>
          <Button label="Done" variant="primary" size="sm" id="settings-done" />
        </Dialog.Close>
      </div>
    </Modal>
  );
}

/** What About's Licenses lists: whose work Emmet builds on, and the provider notice (D-37). */
const LICENSES: [string, string][] = [
  ["LDraw parts library", "Parts from LDraw.org, by the LDraw part authors, under CC BY 4.0 (older parts CC BY 2.0). Emmet uses the files as they are and doesn't change them. LDraw is a trademark of the Estate of James Jessiman."],
  ["Rebrickable", `${REBRICKABLE_CREDIT}, from its catalog downloads.`],
  ["LDCad Shadow Library", "Emmet's connector table is converted from the LDCad Shadow Library by Roland Melkert and its contributors, under CC BY-SA 4.0, and kept under that license."],
  ["Icons", "Lucide icons, under the ISC License (some from Feather, under the MIT License)."],
  ["Your agent", "Claude Code and Codex are made by Anthropic and OpenAI, not by Emmet. Your plan and its limits are between you and them, and you use them with Emmet at your own risk. How your agent handles your data is between you and its provider."],
  ["Emmet Builder", "Free and open source under the AGPL-3.0; the modeling library and templates under the Apache License 2.0. The models you make are yours."],
];

export function About({ info, theme, onClose }: { info: AppInfo | null; theme: "light" | "dark"; onClose: () => void }) {
  const [licenses, setLicenses] = useState(false);
  return (
    <Modal title="About Emmet Builder" id="about" className="about" onClose={onClose}>
      {licenses ? (
        <div className="licenses" id="licenses">
          <p className="dialog-title">Licenses</p>
          {LICENSES.map(([title, text]) => (
            <p key={title}>
              <b>{title}.</b> {text}
            </p>
          ))}
        </div>
      ) : (
        <>
          <img className="about-icon" src={windows ? iconWin : iconMac} alt="" />
          <img className="about-wordmark" src={theme === "dark" ? antDark : antLight} alt="Emmet Builder" />
          <p id="about-versions">
            Version {__EMMET_VERSION__} · LDraw library {info?.library ?? "not installed"} · Parts data {info?.parts_data ?? "none"}
          </p>
          <p>Free and open source under the AGPL-3.0. Your models and chats stay on this computer; Emmet has no accounts and no tracking.</p>
          <p className="small" id="about-connects">
            Emmet connects only to: the LDraw library and parts data downloads, its own update check, and the agent you pick, which talks to Anthropic or OpenAI on your plan.
          </p>
          <p className="small">
            LEGO® is a trademark of the LEGO Group, which does not sponsor, authorize, or endorse this software. LDraw is a trademark of the Estate of James Jessiman. Claude is a trademark of Anthropic, and Codex and ChatGPT of OpenAI; Emmet isn't affiliated with or endorsed by them.
          </p>
        </>
      )}
      <div className="dialog-actions">
        <Button label={licenses ? "Back" : "Licenses"} variant="quiet" size="sm" id="about-licenses" onClick={() => setLicenses((l) => !l)} />
        <Dialog.Close asChild>
          <Button label="Close" size="sm" id="about-close" />
        </Dialog.Close>
      </div>
    </Modal>
  );
}

export function Shortcuts({ onClose }: { onClose: () => void }) {
  return (
    <Modal title="Keyboard shortcuts" id="shortcuts" onClose={onClose}>
      <table className="data-table shortcuts">
        <tbody>
          {KEYS.map((r) => (
            <tr key={r.action}>
              <td>{r.does}</td>
              <td className="num">
                {r.keys.map((k) => (
                  <kbd key={keyName(k)} className="kbd" data-c="kbd">
                    {keyName(k)}
                  </kbd>
                ))}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="dialog-actions">
        <Dialog.Close asChild>
          <Button label="Done" variant="primary" size="sm" />
        </Dialog.Close>
      </div>
    </Modal>
  );
}
