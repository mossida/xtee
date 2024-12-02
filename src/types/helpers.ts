import type { ProceduresDef, RSPCError } from "@rspc/client";
import type {
  UseMutationOptions as UseMutationOptionsTanstack,
  UseQueryOptions as UseQueryOptionsTanstack,
} from "@tanstack/react-query";

export type ProcedureKeys<
  B extends ProceduresDef,
  T extends keyof ProceduresDef
> = B[T]["key"];

export type ProcedureResult<
  B extends ProceduresDef,
  T extends keyof ProceduresDef,
  K extends ProcedureKeys<B, T>
> = Extract<B[T], { key: K }>["result"];

export type ProcedureInput<
  B extends ProceduresDef,
  T extends keyof ProceduresDef,
  K extends ProcedureKeys<B, T>
> = Extract<B[T], { key: K }>["input"] extends never
  ? // biome-ignore lint/suspicious/noConfusingVoidType: <explanation>
    void
  : Extract<B[T], { key: K }>["input"];

export type UseQueryOptions<T extends ProcedureKeys<ProceduresDef, "queries">> =
  Omit<
    UseQueryOptionsTanstack<ProcedureResult<ProceduresDef, "queries", T>>,
    "queryKey" | "queryFn"
  >;

export type UseMutationOptions<
  T extends ProcedureKeys<ProceduresDef, "mutations">
> = Omit<
  UseMutationOptionsTanstack<
    ProcedureResult<ProceduresDef, "mutations", T>,
    RSPCError,
    ProcedureInput<ProceduresDef, "mutations", T>
  >,
  "mutationKey" | "mutationFn"
>;
