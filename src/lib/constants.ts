export const isServer = typeof window === "undefined";
export const isTauri = !isServer && "__TAURI_INTERNALS__" in window;

export function rpmToSpeed(rpm: number, pulses = 800) {
  return Math.round((rpm * pulses) / 60);
}

export function speedToRpm(speed: number, pulses = 800) {
  return Math.round((speed * 60) / pulses);
}
