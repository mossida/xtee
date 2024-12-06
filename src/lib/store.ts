import {
  type DefaultError,
  type UseMutationOptions,
  type UseQueryOptions,
  useMutation as useMutationTanstack,
  useQuery as useQueryTanstack,
} from "@tanstack/react-query";
import { LazyStore, load } from "@tauri-apps/plugin-store";
import { isTauri } from "./constants";

export const storeContainer = isTauri ? await load("store.json") : null;

export type Store = {
  "motor.maxRotations": number;
  "motor.speeds": {
    [key: string]: number;
  };
};

type StoreKey = keyof Store;

export const store = {
  useQuery: <T extends StoreKey>(
    key: T,
    options?: UseQueryOptions<Store[T] | undefined>,
  ) => {
    return useQueryTanstack({
      ...options,
      queryKey: [key],
      queryFn: () => storeContainer?.get<Store[T]>(key),
    });
  },
  useMutation: <T extends StoreKey>(
    key: T,
    options?: UseMutationOptions<void, DefaultError, Store[T]>,
  ) => {
    return useMutationTanstack({
      ...options,
      mutationFn: async (value: Store[T]) => {
        await storeContainer?.set(key, value);
      },
    });
  },
};
