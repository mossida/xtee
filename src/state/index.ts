import { listenEvent } from "@/hooks/use-event";
import type { ActuatorStatus, MotorStatus } from "@/types/bindings";

import { atom } from "jotai";
import { withAtomEffect } from "jotai-effect";
import { atomFamily } from "jotai/utils";

export const actuatorTargetAtom = atom<number>(0);

export const actuatorStatusAtom = withAtomEffect(
  atom<ActuatorStatus | null>(null),
  (_, set) => {
    const promise = listenEvent("actuator-status", (payload) => {
      set(actuatorStatusAtom, payload.data);
    });

    return () => promise.then((unsubscribe) => unsubscribe());
  },
);

export const motorModeAtom = atom<"twisting" | "serving" | "manual">(
  "twisting",
);

export const motorStatusViewAtom = atom<"1" | "2">("1");

export const motorStatusFamily = atomFamily((motor: 1 | 2) =>
  withAtomEffect(atom<MotorStatus | null>(null), (_, set) => {
    const promise = listenEvent("motor-status", (payload) => {
      if (payload.data[0] === motor) {
        set(motorStatusFamily(motor), payload.data[1]);
      }
    });

    return () => promise.then((unsubscribe) => unsubscribe());
  }),
);
