"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { invoke } from "@tauri-apps/api/core";
import { QuantityInput } from "@/components/ui/quantity-input";
import { useState } from "react";
import { useMutation } from "react-query";
import { CurrentLoad } from "@/components/current-load";
import { StepActuator, stopActuatorOptions } from "@/components/step-actuator";
import { ManualActuator } from "@/components/manual-actuator";

export default function Home() {
  const [setpoint, setSetpoint] = useState(0);

  const { mutate: stopActuator } = useMutation(stopActuatorOptions);
  const { mutate: loadActuator } = useMutation({
    mutationFn: async () => {
      await invoke("actuator_load", { setpoint });
    },
  });

  const { mutate: keepActuator } = useMutation({
    mutationFn: async () => {
      await invoke("actuator_keep", { setpoint });
    },
  });

  return (
    <div className="w-full">
      <div className="">
        <Card>
          <CardContent className="flex flex-row justify-between items-stretch p-6">
            <CurrentLoad />

            <div className="w-full flex-grow flex flex-col justify-between px-4">
              <QuantityInput
                min={0}
                max={250}
                value={setpoint}
                onChange={setSetpoint}
              />
              <div className="flex flex-col gap-2">
                <Button size="lg" onClick={() => loadActuator()}>
                  Reach load
                </Button>
                <Button size="lg" onClick={() => keepActuator()}>
                  Keep loaded
                </Button>
              </div>
            </div>

            <div className="w-full flex-grow flex flex-col justify-between px-4">
              <ManualActuator />
            </div>

            <div className="w-full flex-grow flex flex-col justify-between px-4 gap-2">
              <Button
                variant="destructive"
                className="flex-grow hover:bg-destructive"
                onClick={() => stopActuator()}
              >
                STOP
              </Button>
              <Button variant="outline" size="lg">
                Advanced functions
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
