import { FunctionsSelector } from "../functions-selector";
import { Card, CardContent } from "../ui/card";
import { AutoActuator } from "./auto-actuator";
import { CurrentLoad } from "./current-load";
import { ManualActuator } from "./manual-actuator";
import { StopActuator } from "./stop-actuator";

export function ControlActuator() {
  return (
    <Card>
      <CardContent className="flex flex-row justify-between items-stretch p-6">
        <CurrentLoad />

        <AutoActuator />

        <div className="w-full flex-grow">
          <ManualActuator />
        </div>

        <div className="w-full flex-grow flex flex-col justify-between px-4 gap-2">
          <StopActuator />
          <FunctionsSelector />
        </div>
      </CardContent>
    </Card>
  );
}
