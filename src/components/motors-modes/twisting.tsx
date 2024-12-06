import { api } from "@/lib/client";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm, useWatch } from "react-hook-form";
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
  direction: z.enum(["mode-1", "mode-2"]),
  speed: z.enum(["slow", "fast"]),
  rotations: z.number().min(1),
});

export function TwistingMode() {
  const { mutate: spin } = api.useMutation("motor/spin");
  const { mutate: keep } = api.useMutation("motor/keep");
  const { mutate: stop } = api.useMutation("motor/stop");

  const form = useForm<z.infer<typeof schema>>({
    defaultValues: {
      direction: "mode-1",
      speed: "slow",
      rotations: 1,
    },
    resolver: zodResolver(schema),
  });

  const mode = useWatch({ control: form.control, name: "direction" });

  const start = () => {
    const values = form.getValues();
    const payload = {
      direction: values.direction === "mode-1" ? 0x01 : 0x00,
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
        direction: values.direction === "mode-1" ? 0x01 : 0x00,
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
                  <Select onValueChange={onChange} value={value} {...field}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select a direction" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="mode-1">Mode 1</SelectItem>
                      <SelectItem value="mode-2">Mode 2</SelectItem>
                    </SelectContent>
                  </Select>
                </FormControl>
                <FormDescription>
                  {mode === "mode-1"
                    ? "Motor 1 will rotate clockwise, motor 2 will rotate counterclockwise."
                    : "Motor 1 will rotate counterclockwise, motor 2 will rotate clockwise."}
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
        <MotorsStatus />
      </div>
    </div>
  );
}
