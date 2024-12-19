"use client";

import { ControlActuator } from "@/components/actuator/control-actuator";
import { ControlMotors } from "@/components/motors/control-motors";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/use-toast";
import * as inputDetection from "detect-it";

export default function Home() {
  const { toast } = useToast();

  return (
    <div className="flex flex-col gap-4">
      <Button
        onClick={() =>
          toast({
            title: "Hello",
            description: JSON.stringify(inputDetection),
          })
        }
      >
        Debug
      </Button>
      <ControlActuator />
      <ControlMotors />
    </div>
  );
}
