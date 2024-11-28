"use client";

import { cn } from "@/utils/cn";
import Link from "next/link";
import { usePathname } from "next/navigation";

export function SecondaryMenu({
  items,
  className,
}: {
  items: { label: string; path: string }[];
} & React.HTMLAttributes<HTMLDivElement>) {
  const pathname = usePathname();

  return (
    <nav className={cn("py-4", className)}>
      <ul className="flex space-x-6 text-sm overflow-auto scrollbar-hide">
        {items.map((item) => (
          <Link
            prefetch
            key={item.path}
            href={item.path}
            className={cn(
              "text-[#606060]",
              pathname === item.path &&
                "text-primary font-medium underline underline-offset-8",
            )}
          >
            <span>{item.label}</span>
          </Link>
        ))}
      </ul>
    </nav>
  );
}
