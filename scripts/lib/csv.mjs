// SPDX-License-Identifier: AGPL-3.0-or-later
// A small RFC 4180 CSV reader and writer for the data scripts and spikes (no dependencies).

/** RFC 4180 CSV rows as objects keyed by the header. */
export function csv(text) {
  const rows = [];
  let row = [];
  let cell = "";
  let quoted = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (quoted) {
      if (c === '"' && text[i + 1] === '"') {
        cell += '"';
        i++;
      } else if (c === '"') quoted = false;
      else cell += c;
    } else if (c === '"') quoted = true;
    else if (c === ",") {
      row.push(cell);
      cell = "";
    } else if (c === "\n") {
      row.push(cell.replace(/\r$/, ""));
      rows.push(row);
      row = [];
      cell = "";
    } else cell += c;
  }
  if (cell || row.length) {
    row.push(cell);
    rows.push(row);
  }
  const [head, ...body] = rows;
  const data = body.filter((r) => r.length > 1 || r[0] !== "");
  data.forEach((r, i) => {
    if (r.length !== head.length) throw new Error(`csv: row ${i + 2} has ${r.length} fields, the header has ${head.length}`);
  });
  return data.map((r) => Object.fromEntries(head.map((h, i) => [h, r[i]])));
}

/** One CSV line, quoting a field only when it needs it. */
export function csvLine(fields) {
  return fields.map((f) => (/[",\n\r]/.test(f ?? "") ? `"${String(f).replaceAll('"', '""')}"` : (f ?? ""))).join(",");
}
