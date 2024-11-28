"use client";

import { LoadVisualizer } from "@/components/load-visualizer";
import { webviewQueryOptions } from "@/components/prefetch-provider";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { invoke } from "@tauri-apps/api/core";
import { QuantityInput } from "@/components/ui/quantity-input";
import { Separator } from "@/components/ui/separator";
import { Slider } from "@/components/ui/slider";
import { useEvent } from "@/hooks/use-event";
import { useState } from "react";
import { useMutation, useQuery } from "react-query";

function formatBalancedNumber(num: number): string {
  if (num === 0) return "0.00";
  if (num < 10) return num.toFixed(3); // 0-9: x.xxx
  if (num < 100) return num.toFixed(3); // 10-99: xx.xxx
  return num.toFixed(2); // 100+: xxx.xx
}

export default function Home() {
  const { data: webview } = useQuery(webviewQueryOptions);
  const { data: weight } = useEvent("data:weight");

  const [setpoint, setSetpoint] = useState(0);

  const { mutate: loadActuator } = useMutation({
    mutationFn: async () => {
      await invoke("actuator_load", { setpoint });
    },
  });

  const { mutate: spinMotor } = useMutation({
    mutationFn: async ({
      slave,
      direction,
      rotations,
      speed,
    }: {
      slave: number;
      direction: "forward" | "backward";
      rotations: number;
      speed: number;
    }) => {
      await invoke("motor_spin", {
        slave,
        direction: direction === "forward" ? 1 : 0,
        rotations,
        speed,
      });
    },
  });

  const { mutate: keepActuator } = useMutation({
    mutationFn: async () => {
      await invoke("actuator_keep", { setpoint });
    },
  });

  const { mutate: moveActuator } = useMutation({
    mutationFn: async (direction: "forward" | "backward") => {
      await invoke("actuator_move", {
        direction: direction === "forward" ? 1 : 0,
      });
    },
  });

  const { mutate: stopActuator } = useMutation({
    mutationFn: async () => {
      await invoke("actuator_stop");
    },
  });

  return (
    <div className="w-full">
      <div className="">
        <Card>
          <CardContent className="flex flex-row justify-between items-stretch p-6">
            <div className="flex flex-col items-start w-full">
              <span className="text-sm text-muted-foreground">
                Current load (kg)
              </span>
              <span className="font-mono font-medium text-5xl mb-6 mt-3">
                {formatBalancedNumber(weight ?? 0)}
              </span>
              <LoadVisualizer current={weight ?? 0} max={200} />
            </div>

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
              <Button
                size="lg"
                onMouseDown={() => moveActuator("forward")}
                onMouseUp={() => stopActuator()}
              >
                Step forward
              </Button>
              <Button
                size="lg"
                onMouseDown={() => moveActuator("backward")}
                onMouseUp={() => stopActuator()}
              >
                Step backward
              </Button>
              <Button
                size="lg"
                onClick={() =>
                  spinMotor({
                    slave: 1,
                    direction: "forward",
                    rotations: 10000,
                    speed: 7000,
                  })
                }
              >
                Unload
              </Button>
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
