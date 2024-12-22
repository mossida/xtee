"use client";

import { useDisableDefaults } from "@/hooks/use-disable-defaults";

export function DefaultsProvider({ children }: { children: React.ReactNode }) {
  useDisableDefaults();

  return children;
}
