"use client";

import { ArrowLeft, Power, Settings } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { Button } from "./ui/button";
import { useShutdown } from "@/hooks/use-shutdown";
import { Dialog, DialogTrigger, DialogContent } from "./ui/dialog";
import { DialogTitle } from "@radix-ui/react-dialog";

export function Header() {
  const pathname = usePathname();
  const shutdown = useShutdown();
  const [open, setOpen] = useState(false);

  const handleShutdown = () => {
    setOpen(false);
    shutdown();
  };

  return (
    <header className="flex justify-between items-center p-6 pb-0">
      {pathname === "/" ? (
        <div className="flex items-center space-x-8">
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
              <Button variant="destructive" size="icon">
                <Power className="size-5" />
              </Button>
            </DialogTrigger>
            <DialogContent>
              <div className="space-y-4 p-5">
                <DialogTitle>Shutdown</DialogTitle>
                <h2 className="text-lg font-semibold"></h2>
                <p className="text-sm text-muted-foreground">
                  Are you sure you want to shutdown the system? This will stop
                  all motors and actuators.
                </p>
                <div className="flex justify-end gap-3 pt-2">
                  <Button variant="outline" onClick={() => setOpen(false)}>
                    Cancel
                  </Button>
                  <Button variant="destructive" onClick={handleShutdown}>
                    Shutdown
                  </Button>
                </div>
              </div>
            </DialogContent>
          </Dialog>
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
