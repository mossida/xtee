import { StepActuator } from "./step-actuator";
import { Button } from "./ui/button";

export function ManualActuator() {
  return (
    <div className="h-full flex flex-col items-stretch gap-2">
      <StepActuator className="flex-grow" direction="forward" />
      <StepActuator className="flex-grow" direction="backward" />
      <Button variant="outline" size="lg">
        Reset to zero
      </Button>
    </div>
  );
}
