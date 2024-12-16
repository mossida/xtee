import type { Event as Events } from "@/types/bindings";
import { listen as listenTauri } from "@tauri-apps/api/event";
import { useCallback, useRef, useSyncExternalStore } from "react";

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
    listenTauri<Event<T>>("app:event", ({ payload }) => {
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

export const listenEvent = <T extends EventTypes>(
  event: T,
  callback: (payload: Event<T>) => void,
) => {
  return listenTauri<Event<T>>("app:event", ({ payload }) => {
    if (payload.type === event && "data" in payload) callback(payload);
  });
};

export function useEvent<T extends EventTypes>(event: T) {
  const dataRef = useRef<EventData<T> | null>(null);

  const subscribe = useCallback(
    (callback: () => void) => {
      const promise = listenEvent(event, (payload) => {
        if (payload.type === event) {
          callback();

          if ("data" in payload) dataRef.current = payload.data as EventData<T>;
        }
      });

      return () => promise.then((unlisten) => unlisten());
    },
    [event],
  );

  return useSyncExternalStore(
    subscribe,
    () => dataRef.current,
    () => dataRef.current,
  );
}
