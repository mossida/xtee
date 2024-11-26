"use client";

import Link from "next/link";
import { Button } from "./ui/button";
import { Icons } from "./ui/icons";
import { usePathname } from "next/navigation";

export function Header() {
  const pathname = usePathname();

  return (
    <header className="flex justify-between items-center p-6 pb-0">
      {pathname === "/" ? (
        <div className="text-lg font-medium">XTEE</div>
      ) : (
        <Link href="/">
          <Button variant="outline" className="pl-2">
            <Icons.ArrowLeft className="size-5" />
            Back
          </Button>
        </Link>
      )}
      {pathname !== "/settings" && (
        <Link href="/settings">
          <Button variant="outline" size="icon">
            <Icons.Settings className="size-5" />
          </Button>
        </Link>
      )}
    </header>
  );
}
