# ADR 0014: Native PDF instructions

Status: accepted, 2026-11-05 (session 4E)

## Context

Plan section 06 leaves the PDF engine to 4E: "webview print-to-PDF or a Rust PDF crate", proven on both OSes, with font
embedding and file size measured, and no new dependency without Barry (section 19, rule 6). The pictures must come
from `viewer.render` (section 13), so the webview draws them whatever writes the file. Section 08's step rules say what
a page shows.

## Options

1. **Webview print-to-PDF.** Tauri 2.12 and wry 0.57 expose no print-to-PDF. WKWebView's `createPDF` (macOS 11+) makes
   one page as tall as the document, so pages would need PDFKit to cut and join them; WebView2's `PrintToPdfAsync`
   paginates by CSS `@page`. Two native code paths (objc2 and webview2-com called directly, new direct dependencies),
   each laid out by its own engine, so the two OSes' PDFs differ; the system font is embedded, which needs a license
   read for SF Pro and Segoe UI. Rejected.
2. **A Rust PDF crate** (`pdf-writer`, `printpdf`). A new dependency for what this book needs: pages, rectangles, lines,
   text in one font, and JPEG images. Rejected: option 3 does that in about 300 lines.
3. **Emmet writes the PDF itself** (chosen). `emmet_core::instructions::pdf`: PDF 1.4 objects and a cross-reference
   table, each page a content stream, the pictures as the webview's JPEGs unchanged (`DCTDecode`, no decoding), text in
   Helvetica and Helvetica-Bold.

## Decisions

- **The webview draws, Rust lays out and writes.** `instructions::jobs` lists the pictures from `steps.json` and
  `placements.json`; `apps/desktop/src/instructions.ts` draws them with one hidden viewer (the app's main window, or
  render mode's hidden window for `emmet export --format pdf`) and returns JPEGs with what the layout needs (crops, the
  scale, turns, where each sub-assembly copy goes, arrows, zoom boxes); `instructions::layout` makes a display list;
  `pdf` writes it. The same display list is what the E2E page baselines draw, so a page is one thing on both OSes.
- **No font is embedded.** Helvetica and Helvetica-Bold are two of PDF's standard 14 fonts, which every reader provides
  (as Helvetica, or Arial, Liberation Sans, or Nimbus Sans, all metric-compatible); the layout measures text with
  Adobe's AFM widths (`instructions::font`). Text is numbers, counts, part ids, color names, and the model's name, in
  WinAnsiEncoding (characters outside it print as "?"). Embedded fonts: 0 bytes. Embedding one would mean a font file
  in the repo (an OFL face, a license note) and subsetting or about 300 KB a book; nothing on these pages needs it.
- **JPEG, not PNG.** A PNG needs inflating to split its alpha, and the code has no inflate of its own; the webview's
  JPEG goes in as it is. Each picture is drawn on the color it sits on (the page, a sub-assembly's box, a parts box),
  so there's no alpha. Quality 0.7, and no more pixels than about 150 dpi at the printed size.
- **The camera follows the work** (section 08: "the camera stays put through a run of steps"): a run of steps shares a
  frame, the steps' new parts with room around them, or the whole build when that's not much more; the run ends when
  the frame would grow past 1.6 times its first size, or when a step wants a much closer look. A step whose new parts
  are out of sight starts a run that may turn the model (the turn icon). A close frame's cut edges stay sharp: faded
  ones made the parts along them look blurred in print (Barry's printout check, 2026-11-05).
- **Render mode's window opts out of WebKit's background throttling**
  (`background_throttling(BackgroundThrottlingPolicy::Disabled)`, macOS 14+). A hidden page's process runs at
  background priority, and a book's 400 pictures crawled at one every few seconds; a single render never noticed.

## Results

macOS (Apple M5 Max, debug build, `emmet export --format pdf` through render mode, 2026-11-05):

| Model | Parts | Steps | Pages | Time | File |
|---|---|---|---|---|---|
| Rung 1, the Cottage | 903 | 160 | 106 (Letter) | 15 s | 30 MB |
| Rung 2, the station (`rung2-claude`) | 289 | 78 | 49 (A4) | 8.5 s | 15 MB |

Most of a file is the step pictures (each about 150 to 270 KB): studs compress poorly. Official instruction PDFs of
sets this size run 15 to 40 MB. Preview and PDFKit open every page; the read-back test (`tests/it/tests/instructions.rs`)
checks the cross-reference table, the pages, and that every picture is a JPEG.

Windows (the hosted runner, no GPU: WebView2 draws in software, 2026-11-05 and 06, manual runs under D-43): the
read-back tests pass, the Cottage's export included, through render mode (run 42122932036), and the app's Export →
PDF saves the Cottage in 171 s and rung 2 in 88 s (run 42159781016). The PDF is written by the same Rust code, in the
same base-14 fonts, with no native path of its own; what differs is how long the webview takes to draw. Windows' page
baselines and its time on real hardware wait for the Windows pass (Barry, 2026-11-06).

## Rerun

```sh
just build-app   # or the E2E build
emmet export --format pdf --paper a4 --project <dir>
cargo test -p emmet-it --test it -- instructions:: --test-threads=1 --nocapture
just e2e instructions
```
