"use client";

import { GeneralSettings } from "@/components/actuator/general-settings";
import { PidSettings } from "@/components/actuator/pid-settings";

export default function ActuatorSettings() {
  // const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="grid grid-cols-2 gap-4">
      {/* <TuneActuatorModal open={isOpen} onOpenChange={setIsOpen} /> */}
      <GeneralSettings />
      <PidSettings />
      {/* <TunerSettings /> */}
    </div>
  );
}
