import "@/styles/globals.css";
import { Header } from "@/components/header";
import { cn } from "@/lib/cn";
import { GeistMono } from "geist/font/mono";
import { GeistSans } from "geist/font/sans";
import type { Viewport } from "next";
import { Providers } from "./providers";
import { Toaster } from "@/components/ui/sonner";

export const viewport: Viewport = {
  initialScale: 1,
  maximumScale: 3,
  minimumScale: 0.5,
  userScalable: false,
  width: "device-width",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html
      lang="en"
      className={cn(`${GeistSans.variable} ${GeistMono.variable}`)}
      suppressHydrationWarning
    >
      <body>
        <Providers>
          <Header />
          <div className="p-6 h-full">{children}</div>
        </Providers>
        <Toaster richColors />
      </body>
    </html>
  );
}
