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
import { rpmToSpeed, speedToRpm } from "@/lib/constants";
import { store } from "@/lib/store";
import { isOverloadedAtom, motorStatusFamily } from "@/state";
import type { MotorMovement } from "@/types/bindings";
import { zodResolver } from "@hookform/resolvers/zod";
import { useAtomValue } from "jotai";
import { useForm } from "react-hook-form";
import { capitalize } from "remeda";
import { z } from "zod";
import { MotorsStatus } from "../motors-status";

const directions = ["clockwise", "counterclockwise"] as const;
const directionItems = directions.map((direction) => ({
  id: direction,
  label: capitalize(direction),
}));

const schema = z.object({
  motor1: z.object({
    direction: z.enum(directions),
    speed: z.number().min(1),
    rotations: z.number().min(1),
  }),
  motor2: z.object({
    direction: z.enum(directions),
    speed: z.number().min(1),
    rotations: z.number().min(1),
  }),
});

function valuesToPayload(values: z.infer<typeof schema>, spp: number) {
  const payload: MotorMovement[] = [];

  for (const motor of [1, 2] as const) {
    const value = values[`motor${motor}`];

    payload.push({
      direction: value.direction === "clockwise",
      speed: rpmToSpeed(value.speed, spp),
      rotations: value.rotations * 10,
    });
  }

  return payload as [MotorMovement, MotorMovement];
}

export function ManualMode() {
  const { data: limits } = store.useQuery("motors.limits");
  const { mutate: spin } = api.useMutation("motor/spin");
  const { mutate: keep } = api.useMutation("motor/keep");
  const { mutate: stop } = api.useMutation("motor/stop");

  const motor1Status = useAtomValue(motorStatusFamily(1));
  const motor2Status = useAtomValue(motorStatusFamily(2));
  const isOverloaded = useAtomValue(isOverloadedAtom);

  const isDisabled =
    isOverloaded || !motor1Status?.status || !motor2Status?.status;

  const spp = limits?.stepsPerPulse ?? 800;
  const form = useForm<z.infer<typeof schema>>({
    defaultValues: {
      motor1: {
        direction: "clockwise",
        speed: 1,
        rotations: 1,
      },
      motor2: {
        direction: "clockwise",
        speed: 1,
        rotations: 1,
      },
    },
    resolver: zodResolver(schema),
  });

  const start = () => {
    const values = form.getValues();
    const payload = valuesToPayload(values, spp);

    spin([1, payload[0]]);
    spin([2, payload[1]]);
  };

  const askStop = () => {
    stop([1, "graceful"]);
    stop([2, "graceful"]);
  };

  const { lock, unlock } = useLockScroll();

  const ref = useLongPress({
    onStart: () => {
      lock();
      const values = form.getValues();
      const payload = valuesToPayload(values, spp);

      keep([1, payload[0]]);
      keep([2, payload[1]]);
    },
    onEnd: () => {
      unlock();
      askStop();
    },
  });

  return (
    <div className="grid grid-cols-4 gap-4">
      <div className="col-span-2">
        <Form {...form}>
          <div className="space-x-4 grid grid-cols-2">
            <div className="space-y-4">
              <h4 className="text-lg font-semibold">Motor 1</h4>
              <FormField
                name="motor1.direction"
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
                        items={directionItems}
                        selectedItem={directionItems.find(
                          (item) => item.id === value,
                        )}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                name="motor1.speed"
                control={form.control}
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Speed</FormLabel>
                    <FormControl>
                      <DialogNumberInput
                        min={1}
                        max={speedToRpm(limits?.maxSpeed ?? 1, spp)}
                        allowFloat={false}
                        allowNegative={false}
                        disabled={isDisabled}
                        {...field}
                      />
                    </FormControl>
                    <FormDescription>
                      Value is in RPM (rotations per minute)
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                name="motor1.rotations"
                control={form.control}
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Rotations</FormLabel>
                    <FormControl>
                      <DialogNumberInput
                        min={1}
                        max={limits?.maxRotations ?? 1000}
                        allowFloat={false}
                        allowNegative={false}
                        disabled={isDisabled}
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>
            <div className="space-y-4">
              <h4 className="text-lg font-semibold">Motor 2</h4>
              <FormField
                name="motor2.direction"
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
                        items={directionItems}
                        selectedItem={directionItems.find(
                          (item) => item.id === value,
                        )}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                name="motor2.speed"
                control={form.control}
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Speed</FormLabel>
                    <FormControl>
                      <DialogNumberInput
                        min={1}
                        max={speedToRpm(limits?.maxSpeed ?? 1, spp)}
                        allowFloat={false}
                        allowNegative={false}
                        disabled={isDisabled}
                        {...field}
                      />
                    </FormControl>
                    <FormDescription>
                      Value is in RPM (rotations per minute)
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                name="motor2.rotations"
                control={form.control}
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Rotations</FormLabel>
                    <FormControl>
                      <DialogNumberInput
                        min={1}
                        max={limits?.maxRotations ?? 1}
                        allowFloat={false}
                        allowNegative={false}
                        disabled={isDisabled}
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>
          </div>
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
          variant="destructive"
          disabled={isDisabled}
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
