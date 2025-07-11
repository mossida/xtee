"use client";

import { api } from "@/lib/client";
import { motorStatusFamily } from "@/state";
import { useAtomValue } from "jotai";
import { Lock } from "lucide-react";
import { useEffect } from "react";
import { Badge } from "../ui/badge";
import { Separator } from "../ui/separator";
import { Toggle } from "../ui/toggle";

export function MotorStatus({ motor }: { motor: 1 | 2 }) {
  const { mutate: setOutputs, data: outputs } =
    api.useMutation("motor/set/outputs");

  const data = useAtomValue(motorStatusFamily(motor));

  return (
    <div className="flex flex-col gap-2">
      <Separator className="my-3" />
      <div className="space-y-4">
        <div className="flex justify-between items-center">
          <span className="text-sm">Axis</span>
          <Toggle
            pressed={outputs}
            className="data-[state=on]:bg-yellow-500 data-[state=on]:text-black rounded-none"
            onPressedChange={(pressed) => setOutputs([motor, pressed])}
          >
            <Lock />
          </Toggle>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm">Condition</span>
          <Badge variant="default" className="text-xs">
            {(data?.status ?? "Unknown").toUpperCase()}
          </Badge>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm">Rotations</span>
          <span className="text-xs">
            {data?.status === "spinning"
              ? (data.data.position / 800).toFixed(2)
              : "N/A"}
          </span>
        </div>
      </div>
    </div>
  );
}
