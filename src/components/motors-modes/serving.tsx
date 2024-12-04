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
  direction: z.enum(["clockwise", "counterclockwise"]),
  speed: z.enum(["slow", "fast"]),
  rotations: z.number().min(1),
});

export function ServingMode() {
  const form = useForm<z.infer<typeof schema>>({
    defaultValues: {
      direction: "clockwise",
      speed: "slow",
      rotations: 1,
    },
    resolver: zodResolver(schema),
  });

  const onSubmit = (data: z.infer<typeof schema>) => {
    console.log(data);
  };

  return (
    <div className="grid grid-cols-3 gap-4">
      <div className="col-span-1">
        <Form {...form}>
          <form className="space-y-4" onSubmit={form.handleSubmit(onSubmit)}>
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
