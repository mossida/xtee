"use client";

import { ArrowLeft, Power, Settings } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Button } from "./ui/button";
import { useShutdown } from "@/hooks/use-shutdown";

export function Header() {
  const pathname = usePathname();
  const shutdown = useShutdown();

  return (
    <header className="flex justify-between items-center p-6 pb-0">
      {pathname === "/" ? (
        <div className="flex items-center space-x-8">
          <Button variant="destructive" size="icon" onClick={() => shutdown()}>
            <Power className="size-5" />
          </Button>
          <div className="text-lg font-medium">XTEE</div>
        </div>
      ) : (
        <Link href="/">
          <Button variant="outline" className="pl-2">
            <ArrowLeft className="size-5 mr-2" />
            Back
          </Button>
        </Link>
      )}
      {!pathname.includes("/settings") && (
        <Link href="/settings/general">
          <Button variant="outline" size="icon">
            <Settings className="size-5" />
          </Button>
        </Link>
      )}
    </header>
  );
}
