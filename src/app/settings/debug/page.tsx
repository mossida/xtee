"use client";

import { JsonViewer } from "@/components/json-viewer";
import { storeContainer } from "@/lib/store";
import { useQuery } from "@tanstack/react-query";
import { getVersion } from "@tauri-apps/api/app";
import * as inputDetection from "detect-it";

export default function DebugPage() {
  const { data: store } = useQuery({
    queryKey: ["debug_store"],
    queryFn: () => storeContainer?.entries(),
  });

  const { data: version } = useQuery({
    queryKey: ["debug_version"],
    queryFn: () => getVersion(),
  });

  const debugObject = {
    version,
    input: { ...inputDetection },
    store: Object.fromEntries(store ?? []),
  };

  return <JsonViewer initialExpanded data={debugObject} />;
}
