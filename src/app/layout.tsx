import "@/styles/globals.css";
import { Header } from "@/components/header";
import { Toaster } from "@/components/ui/toaster";
import { cn } from "@/lib/cn";
import { GeistMono } from "geist/font/mono";
import { GeistSans } from "geist/font/sans";
import { Providers } from "./providers";

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
      <head>
        <script src="https://unpkg.com/react-scan/dist/auto.global.js" async />
      </head>
      <body>
        <Providers>
          <Header />
          <div className="p-6 h-full">{children}</div>
        </Providers>
        <Toaster />
      </body>
    </html>
  );
}
