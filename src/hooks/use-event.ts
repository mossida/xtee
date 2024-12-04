import { listen as listenEvent, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { isTauri } from "@/lib/constants";

export function useEvent<T extends string, P>(event: T) {
  const client = useQueryClient();

  const listen = useCallback(
    <P>(...args: Parameters<typeof listenEvent>) =>
      isTauri
        ? listenEvent<P>(...args)
        : (new Promise((resolve) => resolve(() => {})) as Promise<UnlistenFn>),
    []
  );

  const query = useQuery<P | null>({
    queryKey: ["event", event],
    queryFn: async () => null,
  });

  useEffect(() => {
    const promise = listen<P>(event, ({ payload }) => {
      client.setQueryData(["event", event], payload);
    });

    return () => {
      promise.then((unlisten) => unlisten());
    };
  }, [client, event, listen]);

  return query;
}
