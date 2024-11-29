import { invoke } from "@tauri-apps/api/core";
import { Button, type ButtonProps } from "./ui/button";
import { useMutation } from "react-query";
import { useLongPress } from "use-long-press";

export const stopActuatorOptions = {
  mutationFn: async () => {
    await invoke("actuator_stop");
  },
};

export const moveActuatorOptions = {
  mutationFn: async (direction: "forward" | "backward") => {
    await invoke("actuator_move", { direction });
  },
};

export function StepActuator({
  direction,
  ...props
}: { direction: "forward" | "backward" } & ButtonProps) {
  const { mutate: stopActuator } = useMutation(stopActuatorOptions);
  const { mutate: moveActuator } = useMutation(moveActuatorOptions);

  const bind = useLongPress(() => {}, {
    onStart: () => moveActuator(direction),
    onFinish: () => stopActuator(),
  });

  return (
    <Button size="lg" {...bind()} {...props}>
      Step {direction}
    </Button>
  );
}
