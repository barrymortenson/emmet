// SPDX-License-Identifier: AGPL-3.0-or-later
// The Parts tab (plan section 14, docs/design/frozen): the parts list, each part's picture in its color, its name with
// the color and part number under it, and the count, most first. A part never made in its color says so; unknown stays
// unknown (bom v1, the parts data pack). Clicking a row highlights its parts. Rebrickable's credit under it (section 12).
import { useEffect, useState } from "react";
import type { Bom, BomLine } from "./bindings";
import { partPicture } from "./pictures";

/** Rebrickable's credit, wherever its data shows (plan section 12). */
export const REBRICKABLE_CREDIT = "Part, color, and set data from Rebrickable (rebrickable.com)";

const AVAILABILITY: Record<string, string> = {
  color_unavailable: "Not made in this color",
  part_unavailable: "Not made",
};

export function availabilityLabel(status: string): string | null {
  return AVAILABILITY[status] ?? null;
}

function Picture({ line }: { line: BomLine }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    void partPicture(line.part, line.color).then((s) => live && setSrc(s));
    return () => {
      live = false;
    };
  }, [line.part, line.color]);
  return src ? <img src={src} alt="" /> : <span className="part-pic-empty" aria-hidden="true" />;
}

export function Parts({ bom, selected, onPick }: { bom: Bom | null | undefined; selected: string | null; onPick: (line: BomLine) => void }) {
  if (!bom) return <p className="muted">The parts list is made after each build.</p>;
  if (bom.problem)
    return (
      <p className="notice" role="alert" id="parts-problem">
        The parts list couldn't be made: {bom.problem}
      </p>
    );
  const kinds = new Set(bom.lines.map((l) => l.part)).size;
  return (
    <>
      <div className="row">
        <b id="parts-total">
          {bom.pieces.toLocaleString("en-US")} parts · {kinds} kinds
        </b>
      </div>
      <table className="data-table" data-c="data-table" aria-label="Parts list" id="parts-list">
        <thead>
          <tr>
            <th className="col-pic">
              <span className="visually-hidden">Picture</span>
            </th>
            <th>Part</th>
            <th className="col-qty num" aria-sort="descending">
              Qty
            </th>
          </tr>
        </thead>
        <tbody>
          {bom.lines.map((l) => {
            const key = `${l.part}|${l.color}`;
            const made = availabilityLabel(l.availability);
            return (
              <tr key={key} aria-selected={selected === key ? "true" : undefined} onClick={() => onPick(l)} data-part={l.part}>
                <td className="col-pic">
                  <Picture line={l} />
                </td>
                <td title={`${l.name}, ${l.color_name}`}>
                  <span className="cell-name">{l.name}</span>
                  <span className="cell-sub">
                    {l.color_name} · {l.part}
                    {made && <span className="warn"> · {made}</span>}
                  </span>
                </td>
                <td className="col-qty num">{l.qty}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {bom.parts_data ? (
        <p className="table-foot" id="parts-credit">
          {REBRICKABLE_CREDIT}
        </p>
      ) : (
        <p className="table-foot">Availability arrives with the parts data pack.</p>
      )}
    </>
  );
}
