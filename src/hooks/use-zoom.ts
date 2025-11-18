import { store } from "@/lib/store";
import { getCurrentWebview } from "@tauri-apps/api/webview";

export function useZoom() {
  const { data: zoom } = store.useQuery("interface.zoom");
  const { mutate: setStore } = store.useMutation();

  const setZoom = async (value: number) => {
    const webview = getCurrentWebview();

    await webview.setZoom(value);

    setStore([["interface.zoom", value]]);
  };

  return [zoom ?? 1.0, setZoom] as const;
}
