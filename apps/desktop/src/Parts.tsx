// SPDX-License-Identifier: AGPL-3.0-or-later
// The Parts drawer: the model's parts list, part × color × quantity, and whether each part was ever made in its color
// (bom v1, the parts data pack). Unknown stays "Unknown", never "Not made".
import type { Bom } from "./bindings";

/** Rebrickable's credit, wherever its data shows (plan section 12). */
export const REBRICKABLE_CREDIT = "Part, color, and set data from Rebrickable (rebrickable.com)";

const AVAILABILITY: Record<string, string> = {
  available: "Available",
  color_unavailable: "Not made in this color",
  part_unavailable: "Not made",
  unknown: "Unknown",
};

export function availabilityLabel(status: string): string {
  return AVAILABILITY[status] ?? "Unknown";
}

export function partsLabel(bom: Bom | null | undefined): string {
  return bom ? `Parts · ${bom.pieces}` : "Parts";
}

export function Parts({ bom }: { bom: Bom | null | undefined }) {
  if (!bom) return <p className="muted">The parts list is made after each build.</p>;
  if (bom.problem)
    return (
      <p className="notice" role="alert" id="parts-problem">
        The parts list couldn't be made: {bom.problem}
      </p>
    );
  return (
    <>
      <p className="muted" id="parts-total">
        {bom.pieces} pieces: {bom.parts} different parts in {bom.lines.length} part and color combinations.
      </p>
      <table className="bom">
        <thead>
          <tr>
            <th scope="col">Qty</th>
            <th scope="col">Part</th>
            <th scope="col">Color</th>
            <th scope="col">Name</th>
            <th scope="col">Availability</th>
          </tr>
        </thead>
        <tbody>
          {bom.lines.map((l) => (
            <tr key={`${l.part}|${l.color}`}>
              <td>{l.qty}</td>
              <td className="mono">{l.part}</td>
              <td>{l.color_name}</td>
              <td>{l.name}</td>
              <td>{availabilityLabel(l.availability)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      {bom.parts_data && (
        <p className="muted" id="parts-credit">
          {REBRICKABLE_CREDIT}
        </p>
      )}
    </>
  );
}
