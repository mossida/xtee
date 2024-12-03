import type { Procedures } from "@/types/bindings";
import type {
  ProcedureInput,
  ProcedureKeys,
  ProcedureResult,
  UseMutationOptions,
  UseQueryOptions,
} from "@/types/helpers";
import { createClient, type RSPCError } from "@rspc/client";
import { TauriTransport } from "@rspc/tauri";
import {
  useQuery as useQueryTanstack,
  useMutation as useQueryMutation,
} from "@tanstack/react-query";

export const client = createClient<Procedures>({
  transport: new TauriTransport(),
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
