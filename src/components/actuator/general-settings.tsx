import { cn } from "@/lib/cn";
import { store } from "@/lib/store";
import { zodResolver } from "@hookform/resolvers/zod";
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
  scaleGain: z.number().min(-100).max(100),
  scaleOffset: z.number().min(-100).max(100),
  maxLoad: z.number().min(0).max(500),
  minLoad: z.number().min(0).max(500),
  precision: z.number().min(0).max(10),
});

export function GeneralSettings() {
  const queries = store.useQueries([
    "scale.gain",
    "scale.offset",
    "actuator.maxLoad",
    "actuator.minLoad",
    "actuator.precision",
  ]);

  const isFetching = queries.some((query) => query.isFetching);

  const { mutate, isPending } = store.useMutation();

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

  const save = () => {
    const values = mapValues(form.getValues(), (value) => Number(value));

    mutate([
      ["scale.gain", values.scaleGain],
      ["scale.offset", values.scaleOffset],
      ["actuator.maxLoad", values.maxLoad],
      ["actuator.minLoad", values.minLoad],
      ["actuator.precision", values.precision],
    ]);
  };

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
                        <FormControl>
                          <Input
                            type="number"
                            step={0.000000001}
                            min={-100}
                            max={100}
                            {...field}
                          />
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
                        <FormControl>
                          <Input
                            type="number"
                            step={0.000000001}
                            min={-100}
                            max={100}
                            {...field}
                          />
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
                        <FormLabel>Precision</FormLabel>
                        <FormControl>
                          <Input
                            type="number"
                            step={0.1}
                            min={0}
                            max={10}
                            {...field}
                          />
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
                        <FormControl>
                          <Input
                            type="number"
                            step={1}
                            min={0}
                            max={500}
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
                        <FormControl>
                          <Input
                            type="number"
                            step={1}
                            min={0}
                            max={500}
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
          onClick={save}
        >
          {isPending ? <Spinner size={16} /> : "Save"}
        </Button>
      </CardFooter>
    </Card>
  );
}
