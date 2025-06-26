import { LimitsSettings } from "@/components/motors/limits-settings";
import { SpeedsSettings } from "@/components/motors/speeds-settings";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { InfoIcon } from "lucide-react";

export default function DualMotorSettings() {
  return (
    <div className="space-y-6">
      <Alert variant={"default"}>
        <InfoIcon className="h-4 w-4" />
        <AlertTitle>Important</AlertTitle>
        <AlertDescription>
          Please ensure all settings are within the recommended ranges for
          optimal machine performance and safety.
        </AlertDescription>
      </Alert>
      <div className="grid gap-6 md:grid-cols-2">
        <SpeedsSettings />
        <LimitsSettings />
      </div>
    </div>
  );
}
