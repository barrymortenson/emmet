// SPDX-License-Identifier: AGPL-3.0-or-later
// Emmet's components (docs/design/components.md), as the app builds them: each emits the frozen markup
// (docs/design/frozen/components.js), with its classes, roles, and data-state, so styles/components.css styles it as
// the frozen screens are styled. Where the inventory names a Radix primitive (D-29), it's underneath: focus, keys,
// and ARIA come from it.
import {
  Box,
  Camera,
  Check,
  ChevronDown,
  CircleCheck,
  Clock,
  Copy,
  Download,
  ExternalLink,
  FileText,
  FileUp,
  FolderOpen,
  Gauge,
  Info,
  List,
  Lock,
  LogIn,
  BookOpen,
  Eye,
  FilePen,
  Search,
  type LucideIcon,
  MessageSquarePlus,
  MessagesSquare,
  Minus,
  OctagonAlert,
  Plus,
  RefreshCw,
  RotateCcwClock,
  Save,
  Scan,
  Send,
  Sheet,
  Shuffle,
  SlidersHorizontal,
  Square,
  Terminal,
  TriangleAlert,
  Undo2,
  WifiOff,
  X,
} from "lucide-react";
import { DropdownMenu, Menubar, Progress as RadixProgress, Select, Tooltip as RadixTooltip } from "radix-ui";
import { type ComponentProps, type ReactNode, useEffect, useRef, useState } from "react";
import brick from "./assets/glyphs/brick.svg?raw";
import floating from "./assets/glyphs/floating.svg?raw";
import submodel from "./assets/glyphs/submodel.svg?raw";

const LUCIDE: Record<string, LucideIcon> = {
  "search": Search,
  "file-pen": FilePen,
  "eye": Eye,
  "book-open": BookOpen,
  box: Box,
  camera: Camera,
  check: Check,
  "chevron-down": ChevronDown,
  "circle-check": CircleCheck,
  clock: Clock,
  copy: Copy,
  download: Download,
  "external-link": ExternalLink,
  "file-text": FileText,
  "file-up": FileUp,
  "folder-open": FolderOpen,
  gauge: Gauge,
  info: Info,
  list: List,
  lock: Lock,
  "log-in": LogIn,
  "message-square-plus": MessageSquarePlus,
  "messages-square": MessagesSquare,
  minus: Minus,
  "octagon-alert": OctagonAlert,
  plus: Plus,
  "refresh-cw": RefreshCw,
  "rotate-ccw-clock": RotateCcwClock,
  save: Save,
  scan: Scan,
  send: Send,
  sheet: Sheet,
  shuffle: Shuffle,
  "sliders-horizontal": SlidersHorizontal,
  square: Square,
  terminal: Terminal,
  "triangle-alert": TriangleAlert,
  "undo-2": Undo2,
  "wifi-off": WifiOff,
  x: X,
};

/** Emmet's own glyphs (docs/design/icons.md): their drawing, from the committed SVGs. */
const GLYPHS: Record<string, string> = Object.fromEntries(
  Object.entries({ brick, floating, submodel }).map(([k, svg]) => [k, svg.slice(svg.indexOf(">", svg.indexOf("<svg")) + 1, svg.lastIndexOf("</svg>"))]),
);

export type IconName = keyof typeof LUCIDE | "brick" | "floating" | "submodel";
type Size = "sm" | "lg" | "xl";

/** A lucide icon or an Emmet glyph, in the text color; always aria-hidden (the control around it has the name). */
export function Icon({ name, size }: { name: IconName; size?: Size }) {
  const glyph = GLYPHS[name];
  if (glyph != null) {
    // Our own committed glyphs, never content from a project.
    return <svg className="icon" data-c="icon" data-size={size} viewBox="0 0 24 24" aria-hidden="true" dangerouslySetInnerHTML={{ __html: glyph }} />;
  }
  const L = LUCIDE[name]!;
  return <L className="icon" data-c="icon" data-size={size} aria-hidden="true" />;
}

export const Spinner = ({ label = "Working" }: { label?: string }) => (
  <span className="spinner" data-c="spinner" role="status" aria-label={label} />
);

export const Kbd = ({ k }: { k: string }) => (
  <kbd className="kbd" data-c="kbd">
    {k}
  </kbd>
);

type ButtonProps = Omit<ComponentProps<"button">, "children"> & {
  label: string;
  variant?: "primary" | "quiet";
  size?: "sm" | "lg";
  icon?: IconName;
  kbd?: string;
  chevron?: boolean;
  busy?: boolean;
  /** In a narrow dock the label drops to the tooltip. */
  compact?: boolean;
};

/** A button: primary (one per surface), secondary, or quiet; busy shows a spinner and says what's happening. */
export function Button({ label, variant, size, icon, kbd, chevron, busy, compact, className, title, disabled, ...rest }: ButtonProps) {
  return (
    <button
      type="button"
      // Busy can't be pressed again (a second install, a second download).
      disabled={disabled || busy}
      className={`button${className ? ` ${className}` : ""}`}
      data-c="button"
      data-variant={variant}
      data-size={size}
      data-busy={busy || undefined}
      data-compact={compact || undefined}
      title={title ?? (compact ? label : undefined)}
      {...rest}
    >
      {busy ? <Spinner label={label} /> : icon ? <Icon name={icon} /> : null}
      <span>{label}</span>
      {kbd && <Kbd k={kbd} />}
      {chevron && <Icon name="chevron-down" size="sm" />}
    </button>
  );
}

/** An icon-only button: its label is its tooltip and its accessible name. */
export function IconButton({ icon, label, size, on, ...rest }: Omit<ComponentProps<"button">, "children"> & { icon: IconName; label: string; size?: "sm"; on?: boolean }) {
  return (
    <button type="button" className="icon-button" data-c="icon-button" aria-label={label} title={label} data-size={size} data-state={on ? "on" : undefined} {...rest}>
      <Icon name={icon} />
    </button>
  );
}

/** A short fact in a pill; a button when it does something. */
export function Badge({ text, tone, icon, onClick, title, id }: { text: string; tone?: "warn" | "accent" | "count"; icon?: IconName; onClick?: () => void; title?: string; id?: string }) {
  const body = (
    <>
      {icon && <Icon name={icon} size="sm" />}
      {text}
    </>
  );
  return onClick ? (
    <button type="button" className="badge" data-c="badge" data-tone={tone} title={title} onClick={onClick} id={id}>
      {body}
    </button>
  ) : (
    <span className="badge" data-c="badge" data-tone={tone} title={title} id={id}>
      {body}
    </span>
  );
}

/** A name's longer meaning, after a pause on hover or keyboard focus (Radix Tooltip). */
export function Tip({ text, children }: { text: string; children: ReactNode }) {
  return (
    <RadixTooltip.Root>
      <RadixTooltip.Trigger asChild>{children}</RadixTooltip.Trigger>
      <RadixTooltip.Portal>
        <RadixTooltip.Content className="tooltip" data-c="tooltip" side="bottom" align="start" sideOffset={2}>
          {text}
        </RadixTooltip.Content>
      </RadixTooltip.Portal>
    </RadixTooltip.Root>
  );
}

export type MenuEntry =
  | "-"
  | { label: string }
  | { note: string }
  | { id: string; text: string; icon?: IconName; sub?: string; keys?: string; checked?: boolean; disabled?: boolean; onSelect: () => void };

/** A menu's items, in a Radix DropdownMenu.Content or (`menubar`) a Menubar.Content: each has its own menu parts, and
 * an item from the other one throws. */
export function MenuItems({ items, menubar }: { items: MenuEntry[]; menubar?: boolean }) {
  const M = menubar ? Menubar : DropdownMenu;
  return (
    <>
      {items.map((it, i) => {
        if (it === "-") return <M.Separator key={i} className="menu-separator" />;
        if ("label" in it)
          return (
            <M.Label key={i} className="menu-label">
              {it.label}
            </M.Label>
          );
        if ("note" in it)
          return (
            <p key={i} className="menu-note">
              {it.note}
            </p>
          );
        return (
          <M.Item
            key={it.id}
            className="menu-item"
            data-menu={it.id}
            disabled={it.disabled}
            data-state={it.checked ? "checked" : undefined}
            onSelect={it.onSelect}
          >
            {it.icon && <Icon name={it.icon} />}
            <span>{it.text}</span>
            {it.sub && <span className="small">{it.sub}</span>}
            {it.keys && <span className="menu-key">{it.keys}</span>}
            {it.checked && <Icon name="check" size="sm" />}
          </M.Item>
        );
      })}
    </>
  );
}

/** A select (Radix Select): its value's words on the trigger, the choices in a menu. */
export function Choose({ label, value, items, onChange, id }: { label: string; value: string; items: [string, string][]; onChange: (v: string) => void; id?: string }) {
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

/** One line of text: Other on a question, an edit to a redirect. */
export function TextField({ label, value, onChange, placeholder, id }: { label: string; value: string; onChange: (v: string) => void; placeholder?: string; id?: string }) {
  return <input className="text-field" data-c="text-field" type="text" aria-label={label} placeholder={placeholder} value={value} id={id} onChange={(e) => onChange(e.target.value)} />;
}

/** Something that takes a while: what, how much in words, and the bar (Radix Progress). */
export function Progress({ label, pct, amount, error, id }: { label: string; pct: number; amount: string; error?: boolean; id?: string }) {
  return (
    <div className="progress" data-c="progress" data-state={error ? "error" : pct >= 100 ? "complete" : "loading"} id={id}>
      <span>{label}</span>
      <span className="small num">{amount}</span>
      <RadixProgress.Root className="progress-track" value={pct} max={100} aria-label={label}>
        <RadixProgress.Indicator className="progress-fill" style={{ width: `${pct}%` }} />
      </RadixProgress.Root>
    </div>
  );
}

/** A card that needs you: plain words and the one action that helps. Bad is role alert. */
export function AlertCard({ tone, icon = "info", title, text, actions, foot, id }: { tone?: "bad"; icon?: IconName; title: string; text: string; actions?: ReactNode; foot?: string; id?: string }) {
  return (
    <div className="alert-card" data-c="alert-card" role={tone === "bad" ? "alert" : "status"} data-tone={tone} id={id}>
      <Icon name={icon} size="lg" />
      <div className="alert-card-body">
        <b>{title}</b>
        <p>{text}</p>
        {actions && <div className="row">{actions}</div>}
        {foot && <p className="small">{foot}</p>}
      </div>
    </div>
  );
}

/** Across the top of the viewer: one line, its icon carrying the tone, and its action. */
export function Banner({ kind, icon, title, text, actions }: { kind: string; icon: IconName; title: string; text?: string; actions: ReactNode }) {
  return (
    <div className="banner" data-c="banner" data-kind={kind} role="status" id={`banner-${kind}`}>
      <Icon name={icon} size="lg" />
      <div className="banner-text">
        <b>{title}</b>
        {text && <p>{text}</p>}
      </div>
      <div className="row">{actions}</div>
    </div>
  );
}

/** An agent in first run or Settings: what it is, the plan, and the one button that fixes it. */
export function SetupCard({ ready, icon, title, text, action, id, children }: { ready?: boolean; icon: IconName; title: string; text: string; action?: ReactNode; id?: string; children?: ReactNode }) {
  return (
    <div className="setup-card" data-c="setup-card" data-ready={ready ? "true" : undefined} id={id}>
      <Icon name={icon} size="xl" />
      <div className="setup-card-text">
        <b>{title}</b>
        <p>{text}</p>
        {children}
      </div>
      {action}
    </div>
  );
}

const MIN_DRAG = 4;

/** A separator between two panes (role separator, focusable): drag it, or move it with the arrows (16 px), Home, and
 * End. `value` is the size it controls; `invert` when dragging right (or down) makes it smaller. */
export function Resizable({ orientation, label, value, min, max, onChange, invert }: { orientation: "vertical" | "horizontal"; label: string; value: number; min: number; max: number; onChange: (v: number) => void; invert?: boolean }) {
  const [dragging, setDragging] = useState(false);
  const start = useRef<{ at: number; value: number } | null>(null);
  const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v)));
  const step = 16;
  useEffect(() => {
    if (!dragging) return;
    const move = (e: PointerEvent) => {
      const s = start.current;
      if (!s) return;
      const d = (orientation === "vertical" ? e.clientX : e.clientY) - s.at;
      if (Math.abs(d) >= MIN_DRAG) onChange(clamp(s.value + (invert ? -d : d)));
    };
    const up = () => setDragging(false);
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    return () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
  });
  return (
    <div
      className="resizable"
      data-c="resizable"
      role="separator"
      tabIndex={0}
      aria-orientation={orientation}
      aria-label={label}
      aria-valuenow={value}
      aria-valuemin={min}
      aria-valuemax={max}
      data-state={dragging ? "dragging" : undefined}
      onPointerDown={(e) => {
        start.current = { at: orientation === "vertical" ? e.clientX : e.clientY, value };
        setDragging(true);
      }}
      onKeyDown={(e) => {
        const grow = orientation === "vertical" ? (invert ? "ArrowLeft" : "ArrowRight") : invert ? "ArrowUp" : "ArrowDown";
        const shrink = orientation === "vertical" ? (invert ? "ArrowRight" : "ArrowLeft") : invert ? "ArrowDown" : "ArrowUp";
        const next = e.key === grow ? value + step : e.key === shrink ? value - step : e.key === "Home" ? min : e.key === "End" ? max : null;
        if (next == null) return;
        e.preventDefault();
        onChange(clamp(next));
      }}
    />
  );
}

/** A time of day as the system writes it, with "Today", "Yesterday", or the date: "Today 14:31". */
export function when(seconds: number, now = new Date()): string {
  const d = new Date(seconds * 1000);
  const time = d.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
  const same = (a: Date, b: Date) => a.toDateString() === b.toDateString();
  if (same(d, now)) return `Today ${time}`;
  const y = new Date(now);
  y.setDate(now.getDate() - 1);
  if (same(d, y)) return `Yesterday ${time}`;
  return `${d.toLocaleDateString(undefined, { month: "short", day: "numeric" })} ${time}`;
}

/** Bytes as megabytes, rounded for words like "96 of 150 MB". */
export const mb = (bytes: number) => Math.round(bytes / 1_000_000);
