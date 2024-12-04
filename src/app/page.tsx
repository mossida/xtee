"use client";

import { CurrentLoad } from "@/components/current-load";
import { ManualActuator } from "@/components/manual-actuator";
import { Mode, ModeSelector } from "@/components/mode-selector";
import { TwistingMode } from "@/components/motors-modes/twisting";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { QuantityInput } from "@/components/ui/quantity-input";
import { api } from "@/lib/client";
import { useState } from "react";

import { ManualMode } from "@/components/motors-modes/manual";
import { ServingMode } from "@/components/motors-modes/serving";
import * as TabsPrimitive from "@radix-ui/react-tabs";

export default function Home() {
  const [setpoint, setSetpoint] = useState(0);
  const [mode, setMode] = useState<Mode>("twisting");

  const { mutate: stopActuator } = api.useMutation("actuator/stop");
  const { mutate: loadActuator } = api.useMutation("actuator/load");
  const { mutate: keepActuator } = api.useMutation("actuator/keep");

  return (
    <div className="flex flex-col gap-4">
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

          <div className="w-full flex-grow flex flex-col items-stretch justify-between px-4">
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
      <Card className="flex flex-col flex-grow">
        <TabsPrimitive.Root className="p-6 space-y-6" value={mode}>
          <TabsPrimitive.List className="flex flex-row justify-between">
            <ModeSelector value={mode} onChange={setMode} />
          </TabsPrimitive.List>
          <TabsPrimitive.Content value="twisting">
            <TwistingMode />
          </TabsPrimitive.Content>
          <TabsPrimitive.Content value="serving">
            <ServingMode />
          </TabsPrimitive.Content>
          <TabsPrimitive.Content value="manual">
            <ManualMode />
          </TabsPrimitive.Content>
        </TabsPrimitive.Root>
      </Card>
    </div>
  );
}
