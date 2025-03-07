import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { cn } from "@/lib/utils";
import { store } from "@/lib/store";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { mapValues } from "remeda";
import { z } from "zod";
import { Button } from "../ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "../ui/form";
import { Input } from "../ui/input";

const schema = z.object({
  setpoint: z.number().min(0).max(200),
  relayAmplitude: z.number().min(0).max(200),
});

export function TunerSettings() {
  const queries = store.useQueries([
    "actuator.tuning.setpoint",
    "actuator.tuning.relayAmplitude",
  ]);

  const [setpoint, relayAmplitude] = queries.map((query) => query.data);

  const { mutate, isPending } = store.useMutation();

  const isFetching = queries.some((query) => query.isFetching);

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    values: {
      setpoint: setpoint ?? 0,
      relayAmplitude: relayAmplitude ?? 0,
    },
  });

  const save = () => {
    const values = mapValues(form.getValues(), (value) => Number(value));

    mutate([
      ["actuator.tuning.setpoint", values.setpoint],
      ["actuator.tuning.relayAmplitude", values.relayAmplitude],
    ]);
  };

  return (
    <Card className="flex flex-col">
      <CardHeader>
        <CardTitle>Tuner settings</CardTitle>
      </CardHeader>
      <CardContent className="grow">
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
                <FormField
                  control={form.control}
                  name="setpoint"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Setpoint</FormLabel>
                      <FormControl>
                        <Input type="number" step={0.1} {...field} />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="relayAmplitude"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Relay amplitude</FormLabel>
                      <FormControl>
                        <Input type="number" step={0.1} {...field} />
                      </FormControl>
                    </FormItem>
                  )}
                />
              </div>
            </Form>
          </div>
        </div>
      </CardContent>
      <CardFooter className="flex justify-end gap-4">
        <Button
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
