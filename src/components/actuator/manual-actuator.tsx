"use client";

import { api } from "@/lib/client";
import { actuatorStatusAtom } from "@/state";
import { useAtomValue } from "jotai";
import { Button } from "../ui/button";
import { StepActuator } from "./step-actuator";

export function ManualActuator() {
  const { mutate: unloadActuator } = api.useMutation("actuator/unload");

  const status = useAtomValue(actuatorStatusAtom);
  const type = status?.status;

  return (
    <div className="h-full flex flex-col items-stretch gap-2">
      <StepActuator
        className="grow"
        direction="forward"
        disabled={type !== "idle" && type !== "overloaded"}
      />
      <StepActuator
        className="grow"
        direction="backward"
        disabled={type !== "idle"}
      />
      <Button
        variant="outline"
        size="lg"
        onClick={() => unloadActuator()}
        disabled={type !== "idle"}
      >
        Reset to zero
      </Button>
    </div>
  );
}
