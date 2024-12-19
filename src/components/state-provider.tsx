"use client";

import { Provider } from "jotai";

export function StateProvider({ children }: { children: React.ReactNode }) {
  return <Provider>{children}</Provider>;
}
