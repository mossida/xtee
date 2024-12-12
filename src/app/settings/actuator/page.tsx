"use client";

import { PidSettings } from "@/components/actuator/pid-settings";
import { TunerSettings } from "@/components/actuator/tuner-settings";
import { TuneActuatorModal } from "@/components/modals/tune-actuator";
import { useState } from "react";

export default function ActuatorSettings() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="grid grid-cols-2 gap-4">
      <TuneActuatorModal open={isOpen} onOpenChange={setIsOpen} />
      <PidSettings onOpen={() => setIsOpen(true)} />
      <TunerSettings />
    </div>
  );
}
