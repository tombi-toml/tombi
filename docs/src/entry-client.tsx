// @refresh reload
import { mount, StartClient } from "@solidjs/start/client";

const PRELOAD_ERROR_RELOAD_KEY = "tombi-preload-error-reload-v1";
const PRELOAD_ERROR_RELOAD_WINDOW_MS = 60_000;

interface PreloadErrorReloadMarker {
  path: string;
  createdAt: number;
}

function parsePreloadErrorReloadMarker(
  value: string | null,
): PreloadErrorReloadMarker | undefined {
  if (!value) return;

  try {
    const marker = JSON.parse(value) as Record<string, unknown>;
    if (
      typeof marker.path === "string" &&
      typeof marker.createdAt === "number"
    ) {
      return {
        path: marker.path,
        createdAt: marker.createdAt,
      };
    }
  } catch {
    return;
  }
}

window.addEventListener("vite:preloadError", (event) => {
  const path = `${window.location.pathname}${window.location.search}${window.location.hash}`;
  const now = Date.now();

  try {
    const previousReload = parsePreloadErrorReloadMarker(
      window.sessionStorage.getItem(PRELOAD_ERROR_RELOAD_KEY),
    );
    if (
      previousReload?.path === path &&
      now - previousReload.createdAt < PRELOAD_ERROR_RELOAD_WINDOW_MS
    ) {
      return;
    }

    window.sessionStorage.setItem(
      PRELOAD_ERROR_RELOAD_KEY,
      JSON.stringify({ path, createdAt: now }),
    );
  } catch {
    // Keep the original error when storage is unavailable rather than risk a reload loop.
    return;
  }

  event.preventDefault();
  window.location.reload();
});

const app = document.getElementById("app");
if (!app) throw new Error("Failed to find app element");

mount(() => <StartClient />, app);
