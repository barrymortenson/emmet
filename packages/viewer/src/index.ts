// SPDX-License-Identifier: AGPL-3.0-or-later

/** The viewer's public surface. 1B puts the three.js scene behind it. */
export interface Viewer {
  dispose(): void;
}

/** Mounts a viewer in `container`. Today it renders nothing. */
export function createViewer(container: HTMLElement): Viewer {
  void container;
  return { dispose() {} };
}
