// SPDX-License-Identifier: AGPL-3.0-or-later
// The Parts drawer: the model's parts list, part × color × quantity (bom v0; availability arrives with 3B).
import type { Bom } from "./bindings";

export function partsLabel(bom: Bom | null | undefined): string {
  return bom ? `Parts · ${bom.pieces}` : "Parts";
}

export function Parts({ bom }: { bom: Bom | null | undefined }) {
  if (!bom) return <p className="muted">The parts list is made after each build.</p>;
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
          </tr>
        </thead>
        <tbody>
          {bom.lines.map((l) => (
            <tr key={`${l.part}|${l.color}`}>
              <td>{l.qty}</td>
              <td className="mono">{l.part}</td>
              <td>{l.color_name}</td>
              <td>{l.name}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}
