"use client";

import { useLongPress } from "@/hooks/use-long-press";
import { api } from "@/lib/client";
import { Button, type ButtonProps } from "../ui/button";

export function StepActuator({
  direction,
  ...props
}: { direction: "forward" | "backward" } & ButtonProps) {
  const { mutate: moveActuator } = api.useMutation("actuator/move");
  const { mutate: stopActuator } = api.useMutation("actuator/stop");

  const ref = useLongPress({
    onStart: () => moveActuator(direction === "forward" ? 1 : 0),
    onEnd: () => stopActuator(),
  });

  return (
    <Button ref={ref} size="lg" {...props}>
      Step {direction}
    </Button>
  );
}
