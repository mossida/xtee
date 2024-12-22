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
  useQueries as useQueriesTanstack,
  useMutation as useQueryMutation,
  useQuery as useQueryTanstack,
} from "@tanstack/react-query";
import { isTauri } from "./constants";

const client = createClient<Procedures>({
  transport: isTauri ? new TauriTransport() : new NoOpTransport(),
});

export const api = {
  useQueries: <T extends ProcedureKeys<Procedures, "queries">>(
    keys: readonly (readonly [
      T,
      ProcedureInput<Procedures, "queries", T>,
      UseQueryOptions<string>,
    ])[],
  ) => {
    return useQueriesTanstack({
      queries: keys.map(([key, input, options]) => ({
        ...options,
        queryKey: [key, input],
        // @ts-expect-error
        queryFn: () => client.query([key, input]),
      })),
    });
  },
  useQuery: <T extends ProcedureKeys<Procedures, "queries">>(
    key: T,
    input: ProcedureInput<Procedures, "queries", T>,
    options?: UseQueryOptions<T>,
  ) => {
    return useQueryTanstack<
      ProcedureResult<Procedures, "queries", T>,
      RSPCError,
      ProcedureResult<Procedures, "queries", T>
    >({
      ...options,
      queryKey: [key],
      // @ts-expect-error
      queryFn: () => client.query([key, input]),
    });
  },
  useMutation: <T extends ProcedureKeys<Procedures, "mutations">>(
    key: T,
    options?: UseMutationOptions<T>,
  ) => {
    return useQueryMutation<
      ProcedureResult<Procedures, "mutations", T>,
      RSPCError,
      ProcedureInput<Procedures, "mutations", T>
    >({
      ...options,
      mutationKey: [key],
      // @ts-expect-error
      mutationFn: (input) => {
        // @ts-expect-error
        return client.mutation([key, input]);
      },
    });
  },
};
