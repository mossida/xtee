import type { Controller } from "@/types/bindings";
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
  "controllers.spawn": Controller[];
  "scale.gain": number;
  "scale.offset": number;
  "actuator.maxLoad": number;
  "actuator.minLoad": number;
  "actuator.precision": number;
  "actuator.tuning.setpoint": number;
  "actuator.tuning.relayAmplitude": number;
  "actuator.pid.settings": {
    proportional: number;
    integral: number;
    derivative: number;
  };
};

type StoreKey = keyof Store;

type Mutation<T extends StoreKey> = [key: T, value: Store[T]];

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
  useMutation: (
    options?: UseMutationOptions<void, DefaultError, Mutation<StoreKey>[]>,
  ) => {
    const client = useQueryClient();

    return useMutationTanstack({
      ...options,
      mutationFn: async <M extends Mutation<StoreKey>[]>(value: M) => {
        await Promise.all(
          value.map(([key, value]) => storeContainer?.set(key, value)),
        );

        client.invalidateQueries({ queryKey: value.map(([key]) => key) });
      },
    });
  },
};
