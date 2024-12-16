import { actuatorStatusAtom } from "@/state";
import { useAtomValue } from "jotai";
import { Button } from "../ui/button";
import { StepActuator } from "./step-actuator";

export function ManualActuator() {
  const status = useAtomValue(actuatorStatusAtom);
  const type = status?.status;

  return (
    <div className="h-full flex flex-col items-stretch gap-2">
      <StepActuator
        className="flex-grow"
        direction="forward"
        disabled={type !== "idle"}
      />
      <StepActuator
        className="flex-grow"
        direction="backward"
        disabled={type !== "idle"}
      />
      <Button variant="outline" size="lg" disabled={type !== "idle"}>
        Reset to zero
      </Button>
    </div>
  );
}
