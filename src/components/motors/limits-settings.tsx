"use client";

import { cn } from "@/lib/cn";
import { store } from "@/lib/store";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { mapValues } from "remeda";
import { z } from "zod";
import { DialogNumberInput } from "../dialog-number-input";
import { Button } from "../ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "../ui/card";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "../ui/form";
import { Spinner } from "../ui/spinner";

const schema = z.object({
  maxSpeed: z.number({ coerce: true }).min(1).max(2000),
  maxRotations: z.number({ coerce: true }).min(1).max(10000),
});

type LimitSettings = z.infer<typeof schema>;

export function LimitsSettings() {
  "use no memo";

  const { data: motorsLimits, isFetching } = store.useQuery("motors.limits");

  const { mutateAsync: save } = store.useMutation();

  const form = useForm<LimitSettings>({
    resolver: zodResolver(schema),
    values: motorsLimits ?? {
      maxSpeed: 500,
      maxRotations: 1000,
    },
  });

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: LimitSettings) => {
      const payload = mapValues(data, (value) => Number(value));

      await save([["motors.limits", payload]]);
    },
  });

  return (
    <Card className="flex flex-col w-full">
      <CardHeader>
        <CardTitle>Limits</CardTitle>
        <CardDescription>
          Set maximum speed and rotation limits.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex-grow">
        <div className="grid grid-cols-1 grid-rows-1">
          <div
            className={cn(
              "col-span-1 row-span-1 w-full h-full flex items-center justify-center [grid-area:1/1] transition-opacity duration-300",
              isFetching ? "" : "opacity-0",
            )}
          >
            <Spinner size={32} />
          </div>
          <div
            className={cn(
              "col-span-1 row-span-1 [grid-area:1/1] transition-opacity duration-300 z-10",
              isFetching ? "opacity-50" : "",
            )}
          >
            <Form {...form}>
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <FormField
                    control={form.control}
                    name={"maxSpeed"}
                    render={({ field: { value, ...field } }) => (
                      <FormItem>
                        <FormLabel>Maximum Speed</FormLabel>
                        <FormControl>
                          <div className="flex items-center space-x-2">
                            <DialogNumberInput
                              min={1}
                              max={2000}
                              value={value.toString()}
                              allowFloat={false}
                              allowNegative={false}
                              {...field}
                            />
                            <span className="text-sm text-muted-foreground">
                              RPM
                            </span>
                          </div>
                        </FormControl>
                        <FormDescription>
                          If the maximum limit is lower than one of speeds
                          settings, the motor will use the limit instead.
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <FormField
                    control={form.control}
                    name={"maxRotations"}
                    render={({ field: { value, ...field } }) => (
                      <FormItem>
                        <FormLabel>Maximum Rotations</FormLabel>
                        <FormControl>
                          <DialogNumberInput
                            min={1}
                            max={10000}
                            value={value.toString()}
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
              </div>
            </Form>
          </div>
        </div>
      </CardContent>
      <CardFooter className="flex justify-end">
        <Button
          type="submit"
          size={"lg"}
          disabled={isPending || !form.formState.isDirty}
          onClick={() => mutate(form.getValues())}
        >
          {isPending ? <Spinner size={16} /> : "Save"}
        </Button>
      </CardFooter>
    </Card>
  );
}
