import { api } from "@/lib/client";
import { useLongPress } from "use-long-press";
import { Button, type ButtonProps } from "../ui/button";

export function StepActuator({
  direction,
  ...props
}: { direction: "forward" | "backward" } & ButtonProps) {
  const { mutate: moveActuator } = api.useMutation("actuator/move");
  const { mutate: stopActuator } = api.useMutation("actuator/stop");

  const bind = useLongPress(() => {}, {
    threshold: 0,
    onStart: () => moveActuator(direction === "forward" ? 1 : 0),
    onFinish: () => stopActuator(),
  });

  return (
    <Button size="lg" {...bind()} {...props}>
      Step {direction}
    </Button>
  );
}
