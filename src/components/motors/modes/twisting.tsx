"use client";

import { DialogNumberInput } from "@/components/dialog-number-input";
import { Button } from "@/components/ui/button";
import { ComboboxDropdown } from "@/components/ui/combobox";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { useLockScroll } from "@/hooks/use-lock-scroll";
import { useLongPress } from "@/hooks/use-long-press";
import { api } from "@/lib/client";
import { rpmToSpeed } from "@/lib/constants";
import { type Store, store } from "@/lib/store";
import { isOverloadedAtom, motorStatusFamily } from "@/state";
import { zodResolver } from "@hookform/resolvers/zod";
import { useAtomValue } from "jotai";
import { useForm, useWatch } from "react-hook-form";
import { capitalize } from "remeda";
import { z } from "zod";
import { MotorsStatus } from "../motors-status";
import { twistingSpeeds } from "../speeds-settings";

const modes = ["mode-1", "mode-2"] as const;
const modeItems = modes.map((mode) => ({
  id: mode,
  label: capitalize(mode),
}));

const speedItems = twistingSpeeds.map((speed) => ({
  id: speed,
  label: capitalize(speed),
}));

const schema = z.object({
  mode: z.enum(modes),
  speed: z.enum(twistingSpeeds),
  rotations: z.number().min(1),
});

type Schema = z.infer<typeof schema>;

export function TwistingMode() {
  const queries = store.useQueries(["motors.limits", "motors.speeds"]);

  const [limits, speeds] = queries.map((query) => query.data) as [
    Store["motors.limits"] | null,
    Store["motors.speeds"] | null,
  ];

  const { mutate: spin } = api.useMutation("motor/spin");
  const { mutate: keep } = api.useMutation("motor/keep");
  const { mutate: stop } = api.useMutation("motor/stop");

  const form = useForm<z.infer<typeof schema>>({
    defaultValues: {
      mode: "mode-1",
      speed: "slow",
      rotations: 1,
    },
    resolver: zodResolver(schema),
  });

  const motor1Status = useAtomValue(motorStatusFamily(1));
  const motor2Status = useAtomValue(motorStatusFamily(2));
  const isOverloaded = useAtomValue(isOverloadedAtom);

  const isDisabled =
    isOverloaded || !motor1Status?.status || !motor2Status?.status;

  console.log(form.formState.isValid);

  const spp = limits?.stepsPerPulse ?? 800;
  const mode = useWatch({ control: form.control, name: "mode" });

  const speedToValue = (speed: Schema["speed"]) => {
    return rpmToSpeed(speeds?.twisting[speed] ?? 1, spp);
  };

  const start = () => {
    const values = form.getValues();
    const payload = {
      direction: values.mode === "mode-1",
      speed: speedToValue(values.speed),
      rotations: values.rotations * 5,
    };

    spin([1, payload]);
    spin([2, payload]);
  };

  const { lock, unlock } = useLockScroll();

  const ref = useLongPress({
    onStart: () => {
      lock();
      const values = form.getValues();
      console.log(values);
      const payload = {
        direction: values.mode === "mode-1",
        speed: speedToValue(values.speed),
        rotations: values.rotations,
      };

      keep([1, payload]);
      keep([2, payload]);
    },
    onEnd: () => {
      unlock();
      stop([1, "graceful"]);
      stop([2, "graceful"]);
    },
  });

  return (
    <div className="grid grid-cols-3 gap-4">
      <div className="col-span-1 space-y-4">
        <Form {...form}>
          <FormField
            name="mode"
            control={form.control}
            render={({ field: { onChange, value } }) => (
              <FormItem>
                <FormLabel>Direction</FormLabel>
                <FormControl>
                  <ComboboxDropdown
                    hasSearch={false}
                    disabled={isDisabled}
                    popoverProps={{ className: "!animate-none" }}
                    onSelect={({ id }) => onChange(id)}
                    items={modeItems}
                    selectedItem={modeItems.find((item) => item.id === value)}
                  />
                </FormControl>
                <FormDescription>
                  {mode === "mode-1"
                    ? "Motor 1 will rotate clockwise, motor 2 will rotate counterclockwise."
                    : "Motor 1 will rotate counterclockwise, motor 2 will rotate clockwise."}
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            name="speed"
            control={form.control}
            render={({ field: { onChange, value } }) => (
              <FormItem>
                <FormLabel>Speed</FormLabel>
                <FormControl>
                  <ComboboxDropdown
                    hasSearch={false}
                    disabled={isDisabled}
                    popoverProps={{ className: "!animate-none" }}
                    onSelect={({ id }) => onChange(id)}
                    items={speedItems}
                    selectedItem={speedItems.find((item) => item.id === value)}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            name="rotations"
            control={form.control}
            render={({ field }) => (
              <FormItem>
                <FormLabel>String rotations</FormLabel>
                <FormControl>
                  <DialogNumberInput
                    min={1}
                    max={limits?.maxRotations}
                    allowFloat={false}
                    allowNegative={false}
                    disabled={isDisabled}
                    {...field}
                  />
                </FormControl>
                <FormDescription>
                  One rotation means half rotation for each motor.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
        </Form>
      </div>
      <div className="flex flex-col justify-stretch gap-2 col-span-1">
        <div className="grid grid-cols-2 gap-2">
          <div>
            <Button
              className="w-full h-16"
              onClick={start}
              disabled={isDisabled || !form.formState.isValid}
            >
              Start rotations
            </Button>
          </div>
          <div>
            <Button
              ref={ref}
              className="w-full h-16"
              disabled={isDisabled || !form.formState.isValid}
            >
              Move manually
            </Button>
          </div>
        </div>
        <Button
          className="w-full hover:bg-destructive flex-grow"
          disabled={isDisabled}
          variant="destructive"
          onClick={() => {
            stop([1, "emergency"]);
            stop([2, "emergency"]);
          }}
        >
          STOP
        </Button>
      </div>
      <div className="col-span-1">
        <MotorsStatus />
      </div>
    </div>
  );
}
