export const isServer = typeof window === "undefined";
export const isTauri = !isServer && "__TAURI_INTERNALS__" in window;
