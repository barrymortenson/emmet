// SPDX-License-Identifier: AGPL-3.0-or-later
// Photos attached in the chat (4D): PNG, JPEG, and HEIC (which the webview decodes on macOS only), turned upright,
// scaled so its long edge is at most 2,048 px, and re-encoded as a JPEG of at most 3.75 MiB, the tightest limits that
// fit every provider's (docs/spec/sessions/4D.md). Drawing on a canvas keeps only the pixels, so no EXIF, GPS, XMP, or
// ICC block survives; the host cleans the JPEG again before it saves or sends it (emmet_core::refs).

/** Mirrors emmet_core::refs::MAX_EDGE and MAX_BYTES. */
export const MAX_EDGE = 2048;
export const MAX_BYTES = 3_932_160;
const QUALITIES = [0.86, 0.78, 0.68, 0.56];
const TYPES = ["image/png", "image/jpeg", "image/heic", "image/heif"];
const EXTENSIONS = /\.(png|jpe?g|heic|heif)$/i;

/** What the composer's picker offers (a drop or a paste is held to `isPhoto`). */
export const ACCEPT = [...TYPES, ".heic", ".heif"].join(",");

/** A photo ready for the host: its JPEG as base64, a data URL for the chip, and its size. */
export type Converted = { name: string; jpeg: string; url: string; width: number; height: number };

/** Whether a dropped or pasted file is a photo Emmet takes (HEIC often comes with no type). */
export function isPhoto(f: File): boolean {
  return TYPES.includes(f.type.toLowerCase()) || ((!f.type || f.type === "application/octet-stream") && EXTENSIONS.test(f.name));
}

/** The longest edge at most `max`, keeping the shape; never enlarged. */
export function scaled(w: number, h: number, max = MAX_EDGE): { width: number; height: number } {
  const k = Math.min(1, max / Math.max(w, h));
  return { width: Math.max(1, Math.round(w * k)), height: Math.max(1, Math.round(h * k)) };
}

const readAs = (b: Blob) =>
  new Promise<string>((done, fail) => {
    const r = new FileReader();
    r.onload = () => done(String(r.result));
    r.onerror = () => fail(r.error);
    r.readAsDataURL(b);
  });

/** The photo decoded, upright (createImageBitmap reads the EXIF orientation); an <img> from a data URL when it can't. */
async function decode(file: Blob): Promise<CanvasImageSource & { width: number; height: number }> {
  try {
    return await createImageBitmap(file, { imageOrientation: "from-image" });
  } catch (e) {
    console.warn("photos: createImageBitmap failed, trying an <img>", e);
    const img = new Image();
    img.src = await readAs(file);
    await img.decode();
    return Object.assign(img, { width: img.naturalWidth, height: img.naturalHeight });
  }
}

/** `file` as a JPEG within the limits, or an error in plain words. */
export async function convert(file: File): Promise<Converted> {
  let source: Awaited<ReturnType<typeof decode>>;
  try {
    source = await decode(file);
  } catch (e) {
    console.error("photos: can't decode", file.name, e);
    const heic = /\.hei[cf]$/i.test(file.name) || /hei[cf]/i.test(file.type);
    throw new Error(heic ? "Emmet can't read HEIC photos here. Save it as a JPEG or PNG, then attach it again." : "Emmet can't read this photo. Try a JPEG or PNG.", { cause: e });
  }
  const { width, height } = scaled(source.width, source.height);
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const g = canvas.getContext("2d")!;
  // A transparent PNG goes on white, as a JPEG has no transparency.
  g.fillStyle = "#fff";
  g.fillRect(0, 0, width, height);
  g.drawImage(source, 0, 0, width, height);
  if ("close" in source && typeof source.close === "function") source.close();
  for (const q of QUALITIES) {
    const blob = await new Promise<Blob | null>((done) => canvas.toBlob(done, "image/jpeg", q));
    // No blob is the encoder failing (out of memory, a canvas too big), not a photo too detailed.
    if (!blob) throw new Error("Emmet couldn't make this photo ready. Try a smaller one.");
    if (blob.size <= MAX_BYTES) {
      const url = await readAs(blob);
      return { name: file.name || "photo.jpg", jpeg: url.slice(url.indexOf(",") + 1), url, width, height };
    }
  }
  throw new Error("This photo is too detailed to send. Try a smaller one.");
}
