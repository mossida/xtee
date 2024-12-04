import { StepActuator } from "./step-actuator";
import { Button } from "./ui/button";

export function ManualActuator() {
  return (
    <>
      <StepActuator direction="forward" />
      <StepActuator direction="backward" />
      <Button size="lg" variant="secondary">
        Unload
      </Button>
    </>
  );
}
