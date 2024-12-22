"use client";

import { useLockScroll } from "@/hooks/use-lock-scroll";
import { useLongPress } from "@/hooks/use-long-press";
import { api } from "@/lib/client";
import { Button, type ButtonProps } from "../ui/button";

export function StepActuator({
  direction,
  ...props
}: { direction: "forward" | "backward" } & ButtonProps) {
  const { mutate: moveActuator } = api.useMutation("actuator/move");
  const { mutate: stopActuator } = api.useMutation("actuator/stop");

  const { lock, unlock } = useLockScroll();

  const ref = useLongPress({
    onStart: () => {
      lock();
      moveActuator(direction === "forward");
    },
    onEnd: () => {
      unlock();
      stopActuator();
    },
  });

  return (
    <Button ref={ref} size="lg" {...props}>
      Step {direction}
    </Button>
  );
}
