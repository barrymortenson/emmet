// SPDX-License-Identifier: AGPL-3.0-or-later
// The compare view (4D): a photo the user sent, beside the model. The user orbits the model by dragging to line it up
// with the photo; opening or closing the view never moves the camera (the viewer only takes the narrower width).

import { useEffect, useState } from "react";
import { commands } from "./bindings";
import { Icon, IconButton } from "./ui";

export function Compare({ path, onClose }: { path: string; onClose: () => void }) {
  const [url, setUrl] = useState<string | null>(null);
  const [failed, setFailed] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    setUrl(null);
    setFailed(null);
    commands.chatPhoto(path).then(
      (r) => {
        if (!live) return;
        if (r.status === "ok") setUrl(r.data);
        else setFailed(`${r.error.message} ${r.error.hint}`.trim());
      },
      (e) => live && setFailed(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [path]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !e.defaultPrevented && document.activeElement?.closest("#photo-compare")) {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <aside className="photo-compare" data-c="photo-compare" id="photo-compare" aria-label="Your photo, beside the model" data-path={path}>
      <div className="photo-compare-bar">
        <Icon name="image" />
        <span className="photo-compare-hint">Drag the model to line it up with your photo</span>
        <IconButton icon="x" size="sm" label="Close the photo (Esc)" id="photo-compare-close" onClick={onClose} autoFocus />
      </div>
      <div className="photo-compare-frame">
        {url ? <img src={url} alt="Your photo" /> : failed ? <p className="small">{failed}</p> : null}
      </div>
    </aside>
  );
}
