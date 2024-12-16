"use client";
import { api } from "@/lib/client";
import { actuatorStatusAtom } from "@/state";
import { useAtomValue } from "jotai";
import { useState } from "react";
import { Button } from "../ui/button";
import { QuantityInput } from "../ui/quantity-input";

export function AutoActuator() {
  const status = useAtomValue(actuatorStatusAtom);
  const type = status?.status;

  const [setpoint, setSetpoint] = useState(0);

  const { mutate: loadActuator } = api.useMutation("actuator/load");
  const { mutate: keepActuator } = api.useMutation("actuator/keep");

  return (
    <div className="w-full flex-grow flex flex-col justify-between px-4">
      <QuantityInput
        min={0}
        max={250}
        value={setpoint}
        onChange={setSetpoint}
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
