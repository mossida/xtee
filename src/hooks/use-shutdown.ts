import { api } from "@/lib/client";

export function useShutdown() {
  const { mutate: stop } = api.useMutation("motor/stop");
  const { mutate: stopActuator } = api.useMutation("actuator/stop");
  const { mutate: shutdown } = api.useMutation("system/shutdown");

  return async () => {
    await Promise.all([
      stop([1, "emergency"]),
      stop([2, "emergency"]),
      stopActuator(),
    ]);

    setTimeout(() => shutdown(), 1000);
  };
}
