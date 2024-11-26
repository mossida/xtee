import "@/styles/globals.css";
import { GeistSans } from "geist/font/sans";
import { GeistMono } from "geist/font/mono";
import { cn } from "@/utils/cn";
import { Providers } from "./providers";
import { Header } from "@/components/header";

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
          <div className="p-6">{children}</div>
        </Providers>
      </body>
    </html>
  );
}
