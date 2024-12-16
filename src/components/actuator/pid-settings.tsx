import { api } from "@/lib/client";
import { cn } from "@/lib/cn";
import { store } from "@/lib/store";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { mapValues } from "remeda";
import { z } from "zod";
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
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "../ui/form";
import { Input } from "../ui/input";
import { Spinner } from "../ui/spinner";

const schema = z.object({
  proportional: z.number().min(0).max(10),
  integral: z.number().min(0).max(10),
  derivative: z.number().min(0).max(10),
});

export function PidSettings({ onOpen }: { onOpen?: () => void }) {
  const { data, isFetching } = store.useQuery("actuator.pid.settings");
  const { mutateAsync: save } = store.useMutation("actuator.pid.settings");
  const { mutateAsync: reload } = api.useMutation("actuator/reload/settings");

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    values: data ?? {
      proportional: 0,
      integral: 0,
      derivative: 0,
    },
  });

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: z.infer<typeof schema>) => {
      await save(mapValues(data, (value) => Number(value)));
      await new Promise((resolve) => setTimeout(resolve, 500));
      await reload();
    },
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>PID settings</CardTitle>
      </CardHeader>
      <CardContent>
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
                <div className="flex flex-col gap-2">
                  <FormField
                    control={form.control}
                    name="proportional"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Proportional</FormLabel>
                        <FormControl>
                          <Input type="number" step={0.000001} {...field} />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <FormField
                    control={form.control}
                    name="integral"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Integral</FormLabel>
                        <FormControl>
                          <Input type="number" step={0.000001} {...field} />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <FormField
                    control={form.control}
                    name="derivative"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Derivative</FormLabel>
                        <FormControl>
                          <Input type="number" step={0.000001} {...field} />
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
        {/* <Button
          variant="secondary"
          size={"lg"}
          disabled={isFetching}
          onClick={() => onOpen()}
        >
          Auto Tune
        </Button> */}
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
