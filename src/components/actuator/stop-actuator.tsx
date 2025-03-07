"use client";

import { api } from "@/lib/client";
import { cn } from "@/lib/utils";
import { Button, type ButtonProps } from "../ui/button";

export function StopActuator({ className, ...props }: ButtonProps) {
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
