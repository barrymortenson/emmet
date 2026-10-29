// SPDX-License-Identifier: AGPL-3.0-or-later
// First run's library download, from this machine (plan session 4A): ldraw-mini zipped the way LDraw's complete.zip
// is (the shared data dir's ldraw-mini.zip, which the integration tests made), served over plain HTTP to 127.0.0.1
// with Range requests, so the app's download resumes as it would from library.ldraw.org. A debug build reads
// EMMET_LIBRARY_URL, EMMET_LIBRARY_SHA256, and EMMET_LIBRARY_RELEASE (src-tauri/src/setup.rs), so E2E never downloads
// the real library. `POST /control?drop=N` makes the next download stop after N bytes, as a dropped connection does.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createServer } from "node:http";
import { join } from "node:path";

let server = null;

/** Starts the server on a free port and sets the app's library variables in this process's environment. */
export async function startLibrary(dataDir) {
  const zip = readFileSync(join(dataDir, "ldraw-mini.zip"));
  let drop = null;
  const served = [];
  server = createServer((req, res) => {
    const url = new URL(req.url, "http://127.0.0.1");
    if (url.pathname === "/control") {
      drop = url.searchParams.has("drop") ? Number(url.searchParams.get("drop")) : null;
      res.end(JSON.stringify({ served }));
      return;
    }
    if (url.pathname === "/served") {
      res.end(JSON.stringify({ served }));
      return;
    }
    if (url.pathname !== "/complete.zip") {
      res.writeHead(404).end();
      return;
    }
    const range = /^bytes=(\d+)-$/.exec(req.headers.range ?? "");
    const start = range ? Number(range[1]) : 0;
    served.push({ start });
    const body = zip.subarray(start);
    res.writeHead(range ? 206 : 200, {
      "Content-Length": body.length,
      ...(range ? { "Content-Range": `bytes ${start}-${zip.length - 1}/${zip.length}` } : {}),
    });
    if (drop != null) {
      const n = drop;
      drop = null;
      // The connection drops partway: what was sent stays, and the client sees a short read.
      res.write(body.subarray(0, n), () => res.socket.destroy());
      return;
    }
    res.end(body);
  });
  await new Promise((r) => server.listen(0, "127.0.0.1", r));
  const base = `http://127.0.0.1:${server.address().port}`;
  Object.assign(process.env, {
    EMMET_LIBRARY_URL: `${base}/complete.zip`,
    EMMET_LIBRARY_SHA256: createHash("sha256").update(zip).digest("hex"),
    // Dated, as a pinned release is, so an older one (the old library's banner) compares with it.
    EMMET_LIBRARY_RELEASE: "2026-10",
    EMMET_E2E_LIBRARY_SERVER: base,
  });
  return { base, size: zip.length };
}

export function stopLibrary() {
  server?.close();
  server = null;
}

/** From a spec (the worker): the next download stops after `bytes`. */
export async function dropNext(bytes) {
  await fetch(`${process.env.EMMET_E2E_LIBRARY_SERVER}/control?drop=${bytes}`, { method: "POST" });
}

/** From a spec: the Range starts of each download the server has answered. */
export async function served() {
  return (await (await fetch(`${process.env.EMMET_E2E_LIBRARY_SERVER}/served`)).json()).served;
}
