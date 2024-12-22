"use client";

import { DefaultsProvider } from "@/components/defaults-provider";
import { StateProvider } from "@/components/state-provider";
import { ThemeProvider } from "@/components/theme-provider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState } from "react";

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient());

  return (
    <DefaultsProvider>
      <StateProvider>
        <QueryClientProvider client={queryClient}>
          <ThemeProvider
            attribute="class"
            forcedTheme="dark"
            disableTransitionOnChange
          >
            {children}
          </ThemeProvider>
        </QueryClientProvider>
      </StateProvider>
    </DefaultsProvider>
  );
}
