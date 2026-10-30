// SPDX-License-Identifier: AGPL-3.0-or-later
// The agent's words as the chat shows them (plan sections 14 and 15): a small markdown (paragraphs, lists, headings
// as bold lines, bold, italics, inline code) turned into React elements, so no HTML in a reply ever reaches the DOM as
// markup. A link shows its words, with the address as their tooltip; nothing in a reply navigates the window. A code
// block folds to one line, since the app shows no code unless asked (law 2).
import type { ReactNode } from "react";

/** Inline marks: **bold**, *italic* or _italic_, `code`, and [words](address). */
function inline(text: string, key: string): ReactNode[] {
  const out: ReactNode[] = [];
  const re = /(\*\*([^*]+)\*\*|`([^`]+)`|\[([^\]]+)\]\(([^)\s]+)\)|(?<![\w*])\*([^*\s][^*]*)\*(?!\w)|(?<!\w)_([^_\s][^_]*)_(?!\w))/g;
  let last = 0;
  for (const m of text.matchAll(re)) {
    if (m.index > last) out.push(text.slice(last, m.index));
    const k = `${key}-${m.index}`;
    if (m[2] != null) out.push(<b key={k}>{m[2]}</b>);
    else if (m[3] != null) out.push(<code key={k}>{m[3]}</code>);
    else if (m[4] != null)
      out.push(
        <span key={k} className="md-link" title={m[5]}>
          {m[4]}
        </span>,
      );
    else out.push(<i key={k}>{m[6] ?? m[7]}</i>);
    last = m.index + m[0].length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

type Block = { kind: "p" | "h"; text: string } | { kind: "ul" | "ol"; items: string[] } | { kind: "code"; lines: string[] };

function blocks(src: string): Block[] {
  const out: Block[] = [];
  const lines = src.replaceAll("\r\n", "\n").split("\n");
  let para: string[] = [];
  const flush = () => {
    if (para.length) out.push({ kind: "p", text: para.join(" ") });
    para = [];
  };
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^\s*```/.test(line)) {
      flush();
      const code: string[] = [];
      for (i++; i < lines.length && !/^\s*```/.test(lines[i]!); i++) code.push(lines[i]!);
      out.push({ kind: "code", lines: code });
      continue;
    }
    const item = line.match(/^\s*(?:([-*+])|(\d+)[.)])\s+(.*)$/);
    if (item) {
      flush();
      const kind = item[1] ? "ul" : "ol";
      const prev = out[out.length - 1];
      if (prev && prev.kind === kind) prev.items.push(item[3]!);
      else out.push({ kind, items: [item[3]!] });
      continue;
    }
    const head = line.match(/^\s*#{1,6}\s+(.*)$/);
    if (head) {
      flush();
      out.push({ kind: "h", text: head[1]! });
      continue;
    }
    if (!line.trim()) flush();
    else para.push(line.trim());
  }
  flush();
  return out;
}

/** `src` as paragraphs and lists. */
export function Markdown({ src }: { src: string }) {
  return (
    <>
      {blocks(src).map((b, i) => {
        const k = `b${i}`;
        switch (b.kind) {
          case "p":
            return <p key={k}>{inline(b.text, k)}</p>;
          case "h":
            return (
              <p key={k}>
                <b>{inline(b.text, k)}</b>
              </p>
            );
          case "ul":
          case "ol": {
            const L = b.kind;
            return (
              <L key={k}>
                {b.items.map((t, j) => (
                  <li key={j}>{inline(t, `${k}-${j}`)}</li>
                ))}
              </L>
            );
          }
          case "code":
            return (
              <details key={k} className="md-code">
                <summary>Code · {b.lines.length === 1 ? "1 line" : `${b.lines.length} lines`}</summary>
                <pre>{b.lines.join("\n")}</pre>
              </details>
            );
        }
      })}
    </>
  );
}
