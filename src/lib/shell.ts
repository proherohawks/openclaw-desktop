import { open } from "@tauri-apps/plugin-shell";

/**
 * Open a URL in the user's default browser via Tauri shell API.
 * Falls back to window.open() when running outside Tauri (e.g. dev in browser).
 */
export async function openUrl(url: string): Promise<void> {
  try {
    await open(url);
  } catch {
    // Fallback for dev mode in browser (no Tauri runtime)
    window.open(url, "_blank", "noopener,noreferrer");
  }
}
