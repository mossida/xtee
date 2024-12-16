"use client";

import { DialogNumberInput } from "@/components/dialog-number-input";
import { Button } from "@/components/ui/button";
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
import { motorStatusFamily } from "@/state";
import { zodResolver } from "@hookform/resolvers/zod";
import { useAtomValue } from "jotai";
import { useForm, useWatch } from "react-hook-form";
import { useLongPress } from "use-long-press";
import { z } from "zod";
import { MotorsStatus } from "../motors-status";

const schema = z.object({
  direction: z.enum(["clockwise", "counterclockwise"]),
  speed: z.enum(["slow", "medium", "fast"]),
  rotations: z.number().min(1),
});

function valuesToPayload(motor: 1 | 2, values: z.infer<typeof schema>) {
  const speed =
    values.speed === "slow" ? 1000 : values.speed === "medium" ? 5000 : 15000;

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

    spin([1, valuesToPayload(1, values)]);
    spin([2, valuesToPayload(2, values)]);
  };

  const bind = useLongPress(() => {}, {
    threshold: 0,
    onStart: () => {
      const values = form.getValues();

      keep([1, valuesToPayload(1, values)]);
      keep([2, valuesToPayload(2, values)]);
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
                  <Select onValueChange={onChange} value={value} {...field}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select a direction" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="clockwise">Clockwise</SelectItem>
                      <SelectItem value="counterclockwise">
                        Counterclockwise
                      </SelectItem>
                    </SelectContent>
                  </Select>
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
                  <Select onValueChange={onChange} value={value} {...field}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select a speed" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="slow">Slow</SelectItem>
                      <SelectItem value="medium">Medium</SelectItem>
                      <SelectItem value="fast">Fast</SelectItem>
                    </SelectContent>
                  </Select>
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
                <FormControl>
                  <DialogNumberInput
                    label="Rotations"
                    value={value.toString()}
                    min={1}
                    max={1000}
                    allowFloat={false}
                    allowNegative={false}
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
