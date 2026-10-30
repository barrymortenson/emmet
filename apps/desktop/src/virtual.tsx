// SPDX-License-Identifier: AGPL-3.0-or-later
// A list that puts only the rows in view (and a screen's worth either side) in the DOM, so a chat of 1,000 messages
// scrolls like one of 10 (session 4B). Rows have their own heights: each is measured once it's drawn, and a row not
// drawn yet counts as the average so far. Pinned to the bottom, it stays there as rows arrive, as a chat log does.
import { type ReactNode, useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";

export type Row = { key: string; node: ReactNode };

/** Before any row is measured: about one line of a message. */
const GUESS = 48;

export function VirtualList({ rows, className, id, label }: { rows: Row[]; className: string; id?: string; label: string }) {
  const box = useRef<HTMLDivElement | null>(null);
  const heights = useRef(new Map<string, number>());
  const [view, setView] = useState({ top: 0, height: 0 });
  const [, remeasured] = useState(0);
  /** At the bottom: new rows keep it there. */
  const pinned = useRef(true);
  /** Where the list last scrolled itself: a scroll event there is its own, which doesn't unpin it, as rows measured
   * since can have moved the bottom. */
  const own = useRef<number | null>(null);

  const avg = () => {
    const h = heights.current;
    if (!h.size) return GUESS;
    let sum = 0;
    for (const v of h.values()) sum += v;
    return sum / h.size;
  };
  const guess = avg();
  const offsets: number[] = [];
  let total = 0;
  for (const r of rows) {
    offsets.push(total);
    total += heights.current.get(r.key) ?? guess;
  }
  const over = Math.max(view.height, 400);
  let first = 0;
  while (first < rows.length - 1 && offsets[first + 1]! < view.top - over) first++;
  let last = first;
  while (last < rows.length - 1 && offsets[last + 1]! < view.top + view.height + over) last++;
  const shown = rows.slice(first, last + 1);

  // The view changes only on a scroll or a resize, and only when it moved: setting it from the layout itself would
  // loop, as each guess of a row's height moves the bottom a little.
  const update = () => {
    const el = box.current;
    if (!el) return;
    setView((v) => (Math.abs(v.top - el.scrollTop) < 1 && v.height === el.clientHeight ? v : { top: el.scrollTop, height: el.clientHeight }));
  };
  const onScroll = () => {
    const el = box.current;
    if (!el) return;
    if (own.current == null || Math.abs(el.scrollTop - own.current) >= 2) pinned.current = el.scrollHeight - el.scrollTop - el.clientHeight < 8;
    own.current = null;
    update();
  };

  // Each drawn row's height, as it lays out and whenever it changes (a streaming message grows).
  const observer = useRef<ResizeObserver | null>(null);
  if (!observer.current && typeof ResizeObserver !== "undefined") {
    observer.current = new ResizeObserver((entries) => {
      let changed = false;
      for (const e of entries) {
        const key = (e.target as HTMLElement).dataset.row;
        const h = (e.target as HTMLElement).offsetHeight;
        if (key && h && heights.current.get(key) !== h) {
          heights.current.set(key, h);
          changed = true;
        }
      }
      if (changed) remeasured((n) => n + 1);
    });
  }
  const measure = useCallback((el: HTMLDivElement | null) => {
    if (el) observer.current?.observe(el);
  }, []);

  // Pinned, it follows the bottom as rows arrive and grow, and as the list itself is resized (opening History, dragging
  // its edge); the scroll event that follows updates the view.
  const follow = () => {
    const el = box.current;
    if (el && pinned.current && el.scrollHeight - el.scrollTop - el.clientHeight >= 1) {
      el.scrollTop = el.scrollHeight;
      own.current = el.scrollTop;
    }
  };
  useLayoutEffect(follow);
  useEffect(() => {
    const el = box.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const r = new ResizeObserver(() => {
      follow();
      update();
    });
    r.observe(el);
    return () => r.disconnect();
  }, []);

  return (
    <div className={className} ref={box} onScroll={onScroll} role="log" aria-label={label} aria-live="polite" id={id} data-rows={rows.length} data-drawn={shown.length}>
      <div style={{ height: offsets[first] ?? 0, flex: "none" }} aria-hidden="true" />
      {shown.map((r) => (
        <div key={r.key} data-row={r.key} ref={measure} className="virtual-row">
          {r.node}
        </div>
      ))}
      <div style={{ height: Math.max(0, total - (offsets[last] ?? 0) - (heights.current.get(rows[last]?.key ?? "") ?? guess)), flex: "none" }} aria-hidden="true" />
    </div>
  );
}
