"use client";

import { api } from "@/lib/client";
import { store } from "@/lib/store";
import { actuatorStatusAtom, actuatorTargetAtom } from "@/state";
import { useAtom, useAtomValue } from "jotai";
import { clamp } from "remeda";
import { TouchNumberInput } from "../touch-number-input";
import { Button } from "../ui/button";

export function AutoActuator() {
  const status = useAtomValue(actuatorStatusAtom);
  const type = status?.status;

  const [setpoint, setSetpoint] = useAtom(actuatorTargetAtom);

  const { mutate: loadActuator } = api.useMutation("actuator/load");
  const { mutate: keepActuator } = api.useMutation("actuator/keep");

  const queries = store.useQueries(["actuator.maxLoad", "actuator.minLoad"]);

  const max = queries[0]?.data ?? 250;
  const min = queries[1]?.data ?? 0;

  return (
    <div className="w-full flex-grow flex flex-col justify-between px-4">
      <TouchNumberInput
        min={min}
        max={max}
        value={setpoint}
        onChange={(step) =>
          setSetpoint((setpoint) => clamp(setpoint + step, { min, max }))
        }
      />
      <div className="flex flex-col gap-2">
        <Button
          size="lg"
          onClick={() => loadActuator(setpoint)}
          disabled={type !== "idle"}
        >
          Reach load
        </Button>
        <Button
          size="lg"
          onClick={() => keepActuator(setpoint)}
          disabled={type !== "idle"}
        >
          Keep loaded
        </Button>
      </div>
    </div>
  );
}
