import { useEvent } from "@/hooks/use-event";
import { api } from "@/lib/client";
import { useState } from "react";
import { Button } from "../ui/button";
import { QuantityInput } from "../ui/quantity-input";

export function AutoActuator() {
  const [setpoint, setSetpoint] = useState(0);

  const { data } = useEvent("actuator-status");
  const { mutate: loadActuator } = api.useMutation("actuator/load");
  const { mutate: keepActuator } = api.useMutation("actuator/keep");

  const { status } = data ?? { status: "unknown" };

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
          disabled={status !== "idle"}
        >
          Reach load
        </Button>
        <Button
          size="lg"
          onClick={() => keepActuator(setpoint)}
          disabled={status !== "idle"}
        >
          Keep loaded
        </Button>
      </div>
    </div>
  );
}
