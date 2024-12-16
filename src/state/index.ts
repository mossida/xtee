import { listenEvent } from "@/hooks/use-event";
import type { ActuatorStatus } from "@/types/bindings";

import { atom } from "jotai";
import { withAtomEffect } from "jotai-effect";

export const actuatorStatusAtom = withAtomEffect(
  atom<ActuatorStatus | null>(null),
  (_, set) => {
    const promise = listenEvent("actuator-status", (payload) => {
      set(actuatorStatusAtom, payload.data);
    });

    return () => promise.then((unsubscribe) => unsubscribe());
  },
);
