"use client";

import { ControlActuator } from "@/components/actuator/control-actuator";
import { ControlMotors } from "@/components/motors/control-motors";

export default function Home() {
  return (
    <div className="flex flex-col gap-4">
      <ControlActuator />
      <ControlMotors />
    </div>
  );
}
