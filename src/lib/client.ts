import type { Procedures } from "@/types/bindings";
import type {
  ProcedureInput,
  ProcedureKeys,
  ProcedureResult,
  UseMutationOptions,
  UseQueryOptions,
} from "@/types/helpers";
import { NoOpTransport, type RSPCError, createClient } from "@rspc/client";
import { TauriTransport } from "@rspc/tauri";
import {
  useMutation as useQueryMutation,
  useQuery as useQueryTanstack,
} from "@tanstack/react-query";
import { isTauri } from "./constants";

export const client = createClient<Procedures>({
  transport: isTauri ? new TauriTransport() : new NoOpTransport(),
});

export const api = {
  useQuery: <T extends ProcedureKeys<Procedures, "queries">>(
    key: T,
    options?: UseQueryOptions<T>
  ) => {
    return useQueryTanstack({
      ...options,
      queryKey: [key],
      // @ts-expect-error
      queryFn: () => client.query([key]),
    });
  },
  useMutation: <T extends ProcedureKeys<Procedures, "mutations">>(
    key: T,
    options?: UseMutationOptions<T>
  ) => {
    return useQueryMutation<
      ProcedureResult<Procedures, "mutations", T>,
      RSPCError,
      ProcedureInput<Procedures, "mutations", T>
    >({
      ...options,
      mutationKey: [key],
      mutationFn: (input) => {
        // @ts-expect-error
        return client.mutation([key, input]);
      },
    });
  },
};
