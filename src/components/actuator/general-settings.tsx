import { api } from "@/lib/client";
import { cn } from "@/lib/cn";
import { store } from "@/lib/store";
import { weightAtom } from "@/state";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { useStore } from "jotai";
import { RefreshCcwIcon, Sparkles, WandSparkles } from "lucide-react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { DialogNumberInput } from "../dialog-number-input";
import { Button } from "../ui/button";
import {
  Card,
  CardContent,
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
  scaleGain: z.number().min(-100).max(100),
  scaleOffset: z.number().min(-100).max(100),
  maxLoad: z.number().min(0).max(500),
  minLoad: z.number().min(0).max(500),
  precision: z.number().min(0).max(10),
});

export function GeneralSettings() {
  "use no memo";

  const atomStore = useStore();

  const queries = store.useQueries([
    "scale.gain",
    "scale.offset",
    "actuator.maxLoad",
    "actuator.minLoad",
    "actuator.precision",
  ]);

  const isFetching = queries.some((query) => query.isFetching);
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    values: {
      scaleGain: queries[0]?.data ?? 0,
      scaleOffset: queries[1]?.data ?? 0,
      maxLoad: queries[2]?.data ?? 0,
      minLoad: queries[3]?.data ?? 0,
      precision: queries[4]?.data ?? 0,
    },
  });

  const { mutateAsync: save } = store.useMutation();
  const { mutateAsync: reload } = api.useMutation("actuator/reload/settings");
  const { mutate, isPending } = useMutation({
    mutationFn: async (data: z.infer<typeof schema>) => {
      await save([
        ["scale.gain", data.scaleGain],
        ["scale.offset", data.scaleOffset],
        ["actuator.maxLoad", data.maxLoad],
        ["actuator.minLoad", data.minLoad],
        ["actuator.precision", data.precision],
      ]);
      await reload();
    },
  });

  return (
    <Card className="flex flex-col">
      <CardHeader>
        <CardTitle>General</CardTitle>
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
              <div className="flex flex-col gap-4">
                <div className="flex flex-grow flex-row gap-2">
                  <FormField
                    control={form.control}
                    name="scaleGain"
                    render={({ field }) => (
                      <FormItem className="flex-grow">
                        <FormLabel>Scale gain</FormLabel>
                        <FormDescription>
                          Multiplier applied to load cell readings. Adjusts
                          measurement sensitivity.
                        </FormDescription>
                        <FormControl>
                          <DialogNumberInput min={-10} max={10} {...field} />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <FormField
                    control={form.control}
                    name="scaleOffset"
                    render={({ field }) => (
                      <FormItem className="flex-grow">
                        <FormLabel>Scale offset</FormLabel>
                        <FormDescription>
                          Constant value added to load cell readings. Used for
                          zero calibration.
                        </FormDescription>
                        <FormControl>
                          <div className="inline-flex items-center gap-2">
                            <DialogNumberInput
                              min={-100}
                              max={100}
                              {...field}
                            />
                            <Button
                              variant="outline"
                              size="icon"
                              onClick={() => {
                                field.onChange(
                                  atomStore.get(weightAtom) + field.value,
                                );
                              }}
                            >
                              <WandSparkles className="w-4 h-4" />
                            </Button>
                          </div>
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <FormField
                    control={form.control}
                    name="precision"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Delta</FormLabel>
                        <FormDescription>
                          The minimum amount of load that will be used to detect
                          if the actuator has reached setpoint.
                        </FormDescription>
                        <FormControl>
                          <DialogNumberInput min={0} max={10} {...field} />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>
                <div className="flex flex-grow flex-row gap-2">
                  <FormField
                    control={form.control}
                    name="maxLoad"
                    render={({ field }) => (
                      <FormItem className="flex-grow">
                        <FormLabel>Max Load</FormLabel>
                        <FormDescription>
                          Maximum allowable load value. System will stop if
                          exceeded.
                        </FormDescription>
                        <FormControl>
                          <DialogNumberInput
                            min={0}
                            max={500}
                            allowFloat={false}
                            allowNegative={false}
                            {...field}
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <FormField
                    control={form.control}
                    name="minLoad"
                    render={({ field }) => (
                      <FormItem className="flex-grow">
                        <FormLabel>Min Load</FormLabel>
                        <FormDescription>
                          Minimum allowable load value. System will stop if
                          reached.
                        </FormDescription>
                        <FormControl>
                          <DialogNumberInput
                            min={0}
                            max={500}
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
      <CardFooter className="flex justify-end gap-4">
        <Button
          className=""
          size={"lg"}
          disabled={isFetching || !form.formState.isDirty}
          onClick={() => mutate(form.getValues())}
        >
          {isPending ? <Spinner size={16} /> : "Save"}
        </Button>
      </CardFooter>
    </Card>
  );
}
