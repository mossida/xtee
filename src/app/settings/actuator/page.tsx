"use client";

import { GeneralSettings } from "@/components/actuator/general-settings";
import { PidSettings } from "@/components/actuator/pid-settings";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { InfoIcon } from "lucide-react";

export default function ActuatorSettings() {
  // const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="space-y-6">
      <Alert variant={"destructive"}>
        <InfoIcon className="h-4 w-4" />
        <AlertTitle>Danger</AlertTitle>
        <AlertDescription>
          Those are critical settings for the entire machine. Please don't
          change them unless you know what you are doing.
        </AlertDescription>
      </Alert>
      <div className="grid grid-cols-2 gap-4">
        {/* <TuneActuatorModal open={isOpen} onOpenChange={setIsOpen} /> */}
        <GeneralSettings />
        <PidSettings />
        {/* <TunerSettings /> */}
      </div>
    </div>
  );
}
