"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { QuantityInput } from "@/components/ui/quantity-input";
import { useState } from "react";
import { CurrentLoad } from "@/components/current-load";
import { ManualActuator } from "@/components/manual-actuator";
import { api } from "@/lib/client";

export default function Home() {
  const [setpoint, setSetpoint] = useState(0);

  const { mutate: stopActuator } = api.useMutation("actuator/stop");
  const { mutate: loadActuator } = api.useMutation("actuator/load");
  const { mutate: keepActuator } = api.useMutation("actuator/keep");

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
                <Button size="lg" onClick={() => loadActuator(setpoint)}>
                  Reach load
                </Button>
                <Button size="lg" onClick={() => keepActuator(setpoint)}>
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
