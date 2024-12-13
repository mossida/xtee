import { promisify } from "node:util";
import { isTauri } from "@/lib/constants";
import type { Event as Events } from "@/types/bindings";
import {
  type QueryOptions,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { type UnlistenFn, listen as listenEvent } from "@tauri-apps/api/event";
import { useCallback, useEffect } from "react";

type Event<T extends EventTypes> = Extract<Events, { type: T }>;
type EventTypes = Events["type"];
type EventData<T extends EventTypes> = Event<T> extends { data: unknown }
  ? Event<T>["data"]
  : never;

export const waitEvent = <T extends EventTypes>(
  event: T,
  selector?: (payload: Event<T>) => boolean,
) => {
  return new Promise<EventData<T> | null>((resolve) => {
    listenEvent<Event<T>>("app:event", ({ payload }) => {
      if (
        payload.type === event &&
        "data" in payload &&
        (selector?.(payload) ?? true)
      )
        resolve(payload.data as EventData<T>);
      else resolve(null);
    });
  });
};

export function useEvent<T extends EventTypes>(
  event: T,
  queryOptions?: Omit<
    QueryOptions<EventData<T> | null, Error, EventData<T> | null, ["event", T]>,
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
    const promise = listen<Event<T>>("app:event", ({ payload }) => {
      if (payload.type === event && "data" in payload)
        client.setQueryData(["event", event], payload.data);
    });

    return () => {
      promise.then((unlisten) => unlisten());
    };
  }, [client, event, listen]);

  return query;
}
