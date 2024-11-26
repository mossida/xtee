import type { Event, Payload } from "@/types/event";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { useQuery, useQueryClient } from "react-query";

export function useEvent<T extends Event, P = Payload<T>>(event: T) {
  const client = useQueryClient();

  const query = useQuery<P | undefined>({
    queryKey: ["event", event],
    queryFn: async () => undefined,
  });

  useEffect(() => {
    const promise = listen<P>(event, ({ payload }) => {
      client.setQueryData(["event", event], payload);
    });

    return () => {
      promise.then((unlisten) => unlisten());
    };
  }, [client, event]);

  return query;
}
