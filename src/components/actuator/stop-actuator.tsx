"use client";

import { api } from "@/lib/client";
import { cn } from "@/lib/utils";
import { Button } from "../ui/button";

export function StopActuator({ className, ...props }: React.ComponentProps<"button">) {
  const { mutate: stopActuator } = api.useMutation("actuator/stop");

  return (
    <Button
      variant="destructive"
      className={cn("grow hover:bg-destructive", className)}
      onClick={() => stopActuator()}
      {...props}
    >
      STOP
    </Button>
  );
}
