"use client";

import { useEvent } from "@/hooks/use-event";
import { api } from "@/lib/client";
import { store } from "@/lib/store";
import type { DialogProps } from "@radix-ui/react-dialog";
import { formatBalancedNumber } from "../actuator/current-load";
import { StepActuator } from "../actuator/step-actuator";
import { Alert, AlertDescription, AlertTitle } from "../ui/alert";
import { Button } from "../ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";

export function TuneActuatorModal(props: DialogProps) {
  const weight = useEvent("weight");
  const { data: setpoint } = store.useQuery("actuator.tuning.setpoint");
  const { mutate: tune } = api.useMutation("actuator/tune");

  const limit = (setpoint ?? 0) * 0.15;
  const isSetpoint =
    (weight ?? 0) <= (setpoint ?? 0) + limit &&
    (weight ?? 0) >= (setpoint ?? 0) - limit;

  return (
    <Dialog {...props}>
      <DialogContent>
        <div className="p-4">
          <DialogHeader>
            <DialogTitle>Tune actuator</DialogTitle>

            <DialogDescription className="pb-4">
              Tune your actuator's PID settings to achieve optimal performance.
              This will automatically detect PID values for your actuator using
              relay feedback.
            </DialogDescription>

            <Alert variant="destructive">
              <AlertTitle>Warning</AlertTitle>
              <AlertDescription>
                This will override your current PID settings.
              </AlertDescription>
            </Alert>
          </DialogHeader>

          <div className="flex flex-col gap-1 mt-6">
            <div className="flex flex-row justify-between items-center gap-2">
              <span className="text-[#878787]">Current load (kg)</span>
              <span className="font-mono font-medium text-lg">
                {formatBalancedNumber(weight ?? 0)}
              </span>
            </div>
            <div className="flex flex-row justify-between items-center gap-2">
              <span className="text-[#878787]">Required setpoint (kg)</span>
              <span className="font-mono font-medium text-lg">
                {formatBalancedNumber(setpoint ?? 0)}
              </span>
            </div>
          </div>

          <div className="flex flex-row gap-2 mt-6 w-full">
            <StepActuator
              variant={"ghost"}
              className="flex-grow"
              direction="forward"
              size="lg"
            />
            <StepActuator
              variant={"ghost"}
              className="flex-grow"
              direction="backward"
              size="lg"
            />
          </div>

          <Button className="w-full mt-4" size="lg" onClick={() => tune()}>
            Start
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
