import { isTauri } from "@/lib/constants";
import type { Event } from "@/types/bindings";
import {
  type QueryKey,
  type QueryOptions,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { type UnlistenFn, listen as listenEvent } from "@tauri-apps/api/event";
import { useCallback, useEffect } from "react";

type Payload<T extends Event["type"]> = Extract<Event, { type: T }>;

export function useEvent<T extends Event["type"]>(
  event: T,
  queryOptions?: Omit<
    QueryOptions<
      Payload<T>["data"] | null,
      Error,
      Payload<T>["data"] | null,
      ["event", T]
    >,
    "queryFn" | "queryKey"
  >,
) {
  const client = useQueryClient();

  const listen = useCallback(
    <P>(...args: Parameters<typeof listenEvent<P>>) =>
      isTauri
        ? listenEvent<P>(...args)
        : (new Promise((resolve) => resolve(() => {})) as Promise<UnlistenFn>),
    [],
  );

  const query = useQuery({
    ...queryOptions,
    queryKey: ["event", event],
    queryFn: async () => null,
  });

  useEffect(() => {
    const promise = listen<Payload<T>>("app:event", ({ payload }) => {
      console.log("payload", payload);

      if (payload.type === event)
        client.setQueryData(["event", event], payload.data);
    });

    return () => {
      promise.then((unlisten) => unlisten());
    };
  }, [client, event, listen]);

  return query;
}
