import { api } from "@/lib/client";
import { zodResolver } from "@hookform/resolvers/zod";
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
import { Label } from "../ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";

const schema = z.object({
  direction: z.enum(["clockwise", "counterclockwise"]),
  speed: z.enum(["slow", "fast"]),
  rotations: z.number().min(1),
});

export function TwistingMode() {
  const { mutate: spin } = api.useMutation("motor/spin");
  const { mutate: keep } = api.useMutation("motor/keep");
  const { mutate: stop } = api.useMutation("motor/stop");
  const { mutate: setOutputs } = api.useMutation("motor/set/outputs");

  const form = useForm<z.infer<typeof schema>>({
    defaultValues: {
      direction: "clockwise",
      speed: "slow",
      rotations: 1,
    },
    resolver: zodResolver(schema),
  });

  const start = () => {
    const values = form.getValues();
    const payload = {
      direction: values.direction === "clockwise" ? 0x01 : 0x00,
      speed: values.speed === "slow" ? 1000 : 15000,
      rotations: values.rotations,
    };

    spin([1, payload]);
    spin([2, payload]);
  };

  const bind = useLongPress(() => {}, {
    threshold: 0,
    onStart: () => {
      const values = form.getValues();
      const payload = {
        direction: values.direction === "clockwise" ? 0x01 : 0x00,
        speed: values.speed === "slow" ? 1000 : 15000,
        rotations: values.rotations,
      };

      keep([1, payload]);
      keep([2, payload]);
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
                <FormDescription>
                  Motors will rotate in same direction.
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
                  <Select
                    onValueChange={onChange}
                    defaultValue={value}
                    {...field}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select a speed" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="slow">Slow</SelectItem>
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
        </Form>
      </div>
      <div className="flex flex-col justify-stretch gap-2 col-span-1">
        <div className="grid grid-cols-2 gap-2">
          <div>
            <Button className="w-full h-16" onClick={start}>
              Start rotations
            </Button>
          </div>
          <div>
            <Button className="w-full h-16" {...bind()}>
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
        {/*<MotorsStatus />*/}
        <Button onClick={() => setOutputs([1, true])}>Enable motor 1</Button>
        <Button onClick={() => setOutputs([2, true])}>Enable motor 2</Button>
        <Button onClick={() => setOutputs([1, false])}>Disable motor 1</Button>
        <Button onClick={() => setOutputs([2, false])}>Disable motor 2</Button>
      </div>
    </div>
  );
}
