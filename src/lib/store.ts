import {
  type DefaultError,
  type UseMutationOptions,
  type UseQueryOptions,
  useMutation as useMutationTanstack,
  useQueries as useQueriesTanstack,
  useQueryClient,
  useQuery as useQueryTanstack,
} from "@tanstack/react-query";
import { load } from "@tauri-apps/plugin-store";
import { isTauri } from "./constants";

export const storeContainer = isTauri ? await load("store.json") : null;

export type Store = {
  "scale.gain": number;
  "scale.offset": number;
  "actuator.maxLoad": number;
  "actuator.tuning.setpoint": number;
  "actuator.tuning.relayAmplitude": number;
  "actuator.pid.settings": {
    proportional: number;
    integral: number;
    derivative: number;
  };
};

type StoreKey = keyof Store;

export const store = {
  useQuery: <T extends StoreKey>(
    key: T,
    options?: UseQueryOptions<Store[T] | null | undefined>,
  ) => {
    return useQueryTanstack({
      ...options,
      queryKey: [key],
      queryFn: async () => (await storeContainer?.get<Store[T]>(key)) ?? null,
    });
  },
  useQueries: <T extends StoreKey>(keys: T[]) => {
    return useQueriesTanstack({
      queries: keys.map((key) => ({
        queryKey: [key],
        queryFn: async () => (await storeContainer?.get<Store[T]>(key)) ?? null,
      })),
    });
  },
  useMutation: <T extends StoreKey>(
    key: T,
    options?: UseMutationOptions<void, DefaultError, Store[T]>,
  ) => {
    const client = useQueryClient();

    return useMutationTanstack({
      ...options,
      mutationFn: async (value: Store[T]) => {
        await storeContainer?.set(key, value);

        client.invalidateQueries({ queryKey: [key] });
      },
    });
  },
};
