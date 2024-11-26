"use client";

import { PrefetchProvider } from "@/components/prefetch-provider";
import { ThemeProvider } from "@/components/theme-provider";
import { useState } from "react";
import { QueryClient, QueryClientProvider } from "react-query";

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(() => new QueryClient());

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider
        attribute="class"
        forcedTheme="dark"
        disableTransitionOnChange
      >
        <PrefetchProvider>{children}</PrefetchProvider>
      </ThemeProvider>
    </QueryClientProvider>
  );
}
