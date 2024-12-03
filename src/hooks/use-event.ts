import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";

export function useEvent<T extends string, P>(event: T) {
  const client = useQueryClient();

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
  }, [client, event]);

  return query;
}
