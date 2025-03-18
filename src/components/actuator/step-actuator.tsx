"use client";

import { useLockScroll } from "@/hooks/use-lock-scroll";
import { useLongPress } from "@/hooks/use-long-press";
import { api } from "@/lib/client";
import { Button } from "../ui/button";

export function StepActuator({
  direction,
  ...props
}: { direction: "forward" | "backward" } & Parameters<typeof Button>[0]) {
  const { mutate: moveActuator } = api.useMutation("actuator/move");
  const { mutate: stopActuator } = api.useMutation("actuator/stop");

  const { lock, unlock } = useLockScroll();

  const ref = useLongPress({
    onStart: () => {
      lock();
      moveActuator(direction === "forward" ? "unload" : "load");
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
