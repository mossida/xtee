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
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { api } from "@/lib/client";
import { store } from "@/lib/store";
import type { Store } from "@/lib/store";
import { motorStatusFamily } from "@/state";
import { zodResolver } from "@hookform/resolvers/zod";
import { useAtomValue } from "jotai";
import { useForm, useWatch } from "react-hook-form";
import { useLongPress } from "use-long-press";
import { z } from "zod";
import { MotorsStatus } from "../motors-status";
import { servingSpeeds } from "../speeds-settings";

const schema = z.object({
  direction: z.enum(["clockwise", "counterclockwise"]),
  speed: z.enum(servingSpeeds),
  rotations: z.number().min(1),
});

function valuesToPayload(
  motor: 1 | 2,
  values: z.infer<typeof schema>,
  speeds: Store["motors.speeds"] | null,
) {
  const speed = speeds?.serving[values.speed] ?? 1;

  const directions =
    motor === 1 ? ([0x01, 0x00] as const) : ([0x00, 0x01] as const);
  const direction =
    values.direction === "clockwise" ? directions[0] : directions[1];

  return {
    direction,
    speed,
    rotations: values.rotations,
  };
}

export function ServingMode() {
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
      direction: "clockwise",
      speed: "slow",
      rotations: 1,
    },
    resolver: zodResolver(schema),
  });

  const motor1Status = useAtomValue(motorStatusFamily(1));
  const motor2Status = useAtomValue(motorStatusFamily(2));

  const start = () => {
    const values = form.getValues();

    spin([1, valuesToPayload(1, values, speeds)]);
    spin([2, valuesToPayload(2, values, speeds)]);
  };

  const bind = useLongPress(() => {}, {
    threshold: 0,
    onStart: () => {
      const values = form.getValues();

      keep([1, valuesToPayload(1, values, speeds)]);
      keep([2, valuesToPayload(2, values, speeds)]);
    },
    onFinish: () => {
      stop([1, "graceful"]);
      stop([2, "graceful"]);
    },
  });

  return (
    <div className="grid grid-cols-3 gap-4">
      <div className="col-span-1 space-y-4">
        <Form {...form}>
          <FormField
            name="direction"
            control={form.control}
            render={({ field: { onChange, value, ...field } }) => (
              <FormItem>
                <FormLabel>Direction</FormLabel>
                <FormControl>
                  <ComboboxDropdown
                    hasSearch={false}
                    className="!animate-none"
                    onSelect={({ id }) => onChange(id)}
                    items={[
                      { id: "clockwise", label: "Clockwise" },
                      { id: "counterclockwise", label: "Counterclockwise" },
                    ]}
                  />
                </FormControl>
                <FormDescription>
                  Motors will rotate in the selected direction.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            name="speed"
            control={form.control}
            render={({ field: { onChange, value, ...field } }) => (
              <FormItem>
                <FormLabel>Speed</FormLabel>
                <FormControl>
                  <ComboboxDropdown
                    hasSearch={false}
                    className="!animate-none"
                    onSelect={({ id }) => onChange(id)}
                    items={servingSpeeds.map((speed) => ({
                      id: speed,
                      label: speed,
                    }))}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            name="rotations"
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
              className="w-full h-16"
              {...bind()}
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
