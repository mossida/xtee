"use client";

import { api } from "@/lib/client";
import { motorStatusFamily } from "@/state";
import { useAtomValue } from "jotai";
import { Badge } from "../ui/badge";
import { Separator } from "../ui/separator";
import { MotorLock } from "./motor-lock";

export function MotorStatus({ motor }: { motor: 1 | 2 }) {
  const { mutate: setOutputs } = api.useMutation("motor/set/outputs");

  const [state, currentOutputs] = useAtomValue(motorStatusFamily(motor));

  return (
    <div className="flex flex-col gap-2">
      <Separator className="my-3" />
      <div className="space-y-4">
        <div className="flex justify-between items-start">
          <div className="flex flex-col">
            <span className="text-sm font-medium">Torque lock</span>
            <span className="text-xs text-muted-foreground">
              {currentOutputs
                ? "Axis locked in position"
                : "Free movement enabled"}
            </span>
          </div>
          <MotorLock
            current={currentOutputs}
            onPressedChange={(pressed) => setOutputs([motor, pressed])}
          />
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm">Condition</span>
          <Badge variant="default" className="text-xs">
            {(state?.status ?? "Unknown").toUpperCase()}
          </Badge>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm">Rotations</span>
          <span className="text-xs">
            {state?.status === "spinning"
              ? (state.data.position / 800).toFixed(2)
              : "N/A"}
          </span>
        </div>
      </div>
    </div>
  );
}
