"use client";

import { JsonViewer } from "@/components/json-viewer";
import { storeContainer } from "@/lib/store";
import { useQuery } from "@tanstack/react-query";
import * as inputDetection from "detect-it";

export default function DebugPage() {
  const { data: store } = useQuery({
    queryKey: ["debug_store"],
    queryFn: () => storeContainer?.entries(),
  });

  const debugObject = {
    input: { ...inputDetection },
    store: Object.fromEntries(store ?? []),
  };

  return <JsonViewer data={debugObject} />;
}
