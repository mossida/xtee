"use client";

import { ArrowLeft, Settings } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Button } from "./ui/button";

export function Header() {
  const pathname = usePathname();

  return (
    <header className="flex justify-between items-center p-6 pb-0">
      {pathname === "/" ? (
        <div className="text-lg font-medium">XTEE</div>
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
