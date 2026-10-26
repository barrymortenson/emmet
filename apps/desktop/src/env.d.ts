// SPDX-License-Identifier: AGPL-3.0-or-later

/** The app version from package.json, set by vite.config.ts. */
declare const __EMMET_VERSION__: string;

/** True only in E2E builds (EMMET_E2E=1): exposes `window.__emmet` for the specs (tests/e2e). */
declare const __EMMET_TEST_HOOKS__: boolean;

/** An image Vite bundles: its URL. */
declare module "*.svg" {
  const url: string;
  export default url;
}
