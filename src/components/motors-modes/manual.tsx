import { api } from "@/lib/client";
import { rpmToSpeed } from "@/lib/constants";
import type { MotorMovement } from "@/types/bindings";
import { zodResolver } from "@hookform/resolvers/zod";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { useLongPress } from "use-long-press";
import { z } from "zod";
import { MotorsStatus } from "../motors-status";
import { Button } from "../ui/button";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "../ui/form";
import { Input } from "../ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";

const schema = z.object({
  motor1: z.object({
    direction: z.enum(["clockwise", "counterclockwise"]),
    speed: z.number().min(1),
    rotations: z.number().min(1),
  }),
  motor2: z.object({
    direction: z.enum(["clockwise", "counterclockwise"]),
    speed: z.number().min(1),
    rotations: z.number().min(1),
  }),
});

function valuesToPayload(values: z.infer<typeof schema>) {
  const payload: MotorMovement[] = [];

  for (const motor of [1, 2] as const) {
    payload.push({
      direction:
        values[`motor${motor}`].direction === "clockwise" ? 0x01 : 0x00,
      speed: Math.round(rpmToSpeed(values[`motor${motor}`].speed)),
      rotations: values[`motor${motor}`].rotations,
    });
  }

  return payload as [MotorMovement, MotorMovement];
}

export function ManualMode() {
  const { mutate: spin } = api.useMutation("motor/spin");
  const { mutate: keep } = api.useMutation("motor/keep");
  const { mutate: stop } = api.useMutation("motor/stop");

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

  const bind = useLongPress(() => {}, {
    threshold: 0,
    onStart: () => {
      const values = form.getValues();
      const payload = valuesToPayload(values);

      keep([1, payload[0]]);
      keep([2, payload[1]]);
    },
    onFinish: () => {
      stop([1, "graceful"]);
      stop([2, "graceful"]);
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
                render={({ field: { onChange, value, ...field } }) => (
                  <FormItem>
                    <FormLabel>Direction</FormLabel>
                    <FormControl>
                      <Select
                        onValueChange={onChange}
                        defaultValue={value}
                        {...field}
                      >
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
                      <Input
                        type="number"
                        min={1}
                        {...field}
                        onChange={(e) => {
                          field.onChange(Number(e.target.value));
                        }}
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
                      <Input
                        type="number"
                        min={1}
                        {...field}
                        onChange={(e) => {
                          field.onChange(Number(e.target.value));
                        }}
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
                render={({ field: { onChange, value, ...field } }) => (
                  <FormItem>
                    <FormLabel>Direction</FormLabel>
                    <FormControl>
                      <Select
                        onValueChange={onChange}
                        defaultValue={value}
                        {...field}
                      >
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
                      <Input
                        type="number"
                        min={1}
                        {...field}
                        onChange={(e) => {
                          field.onChange(Number(e.target.value));
                        }}
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
                      <Input
                        type="number"
                        min={1}
                        {...field}
                        onChange={(e) => {
                          field.onChange(Number(e.target.value));
                        }}
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
              disabled={!form.formState.isValid}
            >
              Start rotations
            </Button>
          </div>
          <div>
            <Button
              className="w-full h-16"
              {...bind()}
              disabled={!form.formState.isValid}
            >
              Move manually
            </Button>
          </div>
        </div>
        <Button
          className="w-full hover:bg-destructive flex-grow"
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
