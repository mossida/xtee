import { useEvent } from "@/hooks/use-event";
import { Button } from "../ui/button";
import { StepActuator } from "./step-actuator";

export function ManualActuator() {
  const { data } = useEvent("actuator-status");

  const { status } = data ?? { status: "unknown" };

  return (
    <div className="h-full flex flex-col items-stretch gap-2">
      <StepActuator
        className="flex-grow"
        direction="forward"
        disabled={status !== "idle"}
      />
      <StepActuator
        className="flex-grow"
        direction="backward"
        disabled={status !== "idle"}
      />
      <Button variant="outline" size="lg" disabled={status !== "idle"}>
        Reset to zero
      </Button>
    </div>
  );
}
