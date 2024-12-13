"use client";

import { waitEvent } from "@/hooks/use-event";
import type { ControllerGroup, ControllerStatus } from "@/types/bindings";
import { useQuery } from "@tanstack/react-query";
import { createContext, useContext } from "react";

type Store = Map<ControllerGroup, ControllerStatus>;

const Context = createContext<Store>(new Map());

export function useStatus() {
  return useContext(Context);
}

export function StatusProvider({ children }: { children: React.ReactNode }) {
  const { isPending } = useQuery({
    queryKey: ["_init"],
    queryFn: () => waitEvent("init"),
    staleTime: Number.POSITIVE_INFINITY,
  });

  // if (isPending)
  //   return (
  //     <div className="h-screen w-screen flex items-center justify-center fixed inset-0 bg-background">
  //       <Spinner size={48} />
  //     </div>
  //   );

  return <Context.Provider value={new Map()}>{children}</Context.Provider>;
}
