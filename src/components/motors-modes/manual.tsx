import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
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
  motor1: z.object({
    direction: z.enum(["clockwise", "counterclockwise"]),
    speed: z.enum(["slow", "fast"]),
    rotations: z.number().min(1),
  }),
  motor2: z.object({
    direction: z.enum(["clockwise", "counterclockwise"]),
    speed: z.enum(["slow", "fast"]),
    rotations: z.number().min(1),
  }),
});

export function ManualMode() {
  const form = useForm<z.infer<typeof schema>>({
    defaultValues: {
      motor1: {
        direction: "clockwise",
        speed: "slow",
        rotations: 1,
      },
      motor2: {
        direction: "clockwise",
        speed: "slow",
        rotations: 1,
      },
    },
    resolver: zodResolver(schema),
  });

  const onSubmit = (data: z.infer<typeof schema>) => {
    console.log(data);
  };

  return (
    <div className="grid grid-cols-4 gap-4">
      <div className="col-span-2">
        <Form {...form}>
          <form
            className="space-x-4 grid grid-cols-2"
            onSubmit={form.handleSubmit(onSubmit)}
          >
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
          </form>
        </Form>
      </div>
      <div className="flex flex-col justify-stretch gap-2 col-span-1">
        <div className="grid grid-cols-2 gap-2">
          <div>
            <Button className="w-full h-16">Start rotations</Button>
          </div>
          <div>
            <Button className="w-full h-16">Move manually</Button>
          </div>
        </div>
        <Button
          className="w-full hover:bg-destructive flex-grow"
          variant="destructive"
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
