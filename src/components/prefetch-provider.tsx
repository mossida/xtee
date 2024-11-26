"use client";

import { getStore } from "@tauri-apps/plugin-store";
import { useEffect } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useQueryClient } from "react-query";

export const storeQueryOptions = {
  queryKey: ["store"],
  queryFn: () => getStore("store.json"),
  staleTime: Number.POSITIVE_INFINITY,
};

export const webviewQueryOptions = {
  queryKey: ["webview"],
  queryFn: getCurrentWebview,
};

export function PrefetchProvider({ children }: { children: React.ReactNode }) {
  const client = useQueryClient();

  useEffect(() => {
    client.prefetchQuery(storeQueryOptions);
    client.prefetchQuery(webviewQueryOptions);
  }, [client]);

  return children;
}
