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
import { useLongPress } from "@/hooks/use-long-press";
import { api } from "@/lib/client";
import { rpmToSpeed } from "@/lib/constants";
import { store } from "@/lib/store";
import { motorStatusFamily } from "@/state";
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
    speed: z.number({ coerce: true }).min(1),
    rotations: z.number({ coerce: true }).min(1),
  }),
  motor2: z.object({
    direction: z.enum(directions),
    speed: z.number({ coerce: true }).min(1),
    rotations: z.number({ coerce: true }).min(1),
  }),
});

function valuesToPayload(values: z.infer<typeof schema>) {
  const payload: MotorMovement[] = [];

  for (const motor of [1, 2] as const) {
    const value = values[`motor${motor}`];

    payload.push({
      direction: value.direction === "clockwise" ? 0x01 : 0x00,
      speed: Math.round(rpmToSpeed(Number(value.speed))),
      rotations: Number(value.rotations),
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
    const payload = valuesToPayload(values);

    spin([1, payload[0]]);
    spin([2, payload[1]]);
  };

  const askStop = () => {
    stop([1, "graceful"]);
    stop([2, "graceful"]);
  };

  const ref = useLongPress({
    onStart: () => {
      const values = form.getValues();
      const payload = valuesToPayload(values);

      keep([1, payload[0]]);
      keep([2, payload[1]]);
    },
    onEnd: askStop,
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
                render={({ field: { value, ...field } }) => (
                  <FormItem>
                    <FormLabel>Speed</FormLabel>
                    <FormControl>
                      <DialogNumberInput
                        value={value.toString()}
                        min={1}
                        max={limits?.maxSpeed ?? 1}
                        allowFloat={false}
                        allowNegative={false}
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
                render={({ field: { value, ...field } }) => (
                  <FormItem>
                    <FormLabel>Rotations</FormLabel>
                    <FormControl>
                      <DialogNumberInput
                        value={value.toString()}
                        min={1}
                        max={limits?.maxRotations ?? 1000}
                        allowFloat={false}
                        allowNegative={false}
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
                render={({ field: { value, ...field } }) => (
                  <FormItem>
                    <FormLabel>Speed</FormLabel>
                    <FormControl>
                      <DialogNumberInput
                        min={1}
                        max={limits?.maxSpeed ?? 1}
                        allowFloat={false}
                        allowNegative={false}
                        value={value.toString()}
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
                render={({ field: { value, ...field } }) => (
                  <FormItem>
                    <FormLabel>Rotations</FormLabel>
                    <FormControl>
                      <DialogNumberInput
                        min={1}
                        max={limits?.maxRotations ?? 1}
                        allowFloat={false}
                        allowNegative={false}
                        value={value.toString()}
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
              disabled={
                !form.formState.isValid ||
                !motor1Status?.status ||
                !motor2Status?.status
              }
            >
              Start rotations
            </Button>
          </div>
          <div>
            <Button
              ref={ref}
              className="w-full h-16"
              disabled={
                !form.formState.isValid ||
                !motor1Status?.status ||
                !motor2Status?.status
              }
            >
              Move manually
            </Button>
          </div>
        </div>
        <Button
          className="w-full hover:bg-destructive flex-grow"
          variant="destructive"
          disabled={!motor1Status?.status || !motor2Status?.status}
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
