"use client";

import { useMutation, useQuery } from "@tanstack/react-query";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import { invoke } from "@tauri-apps/api/core";
import type { SerialPort } from "@/types/serial";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";

import type { DialogProps } from "@radix-ui/react-dialog";
import { uniqueBy } from "remeda";

export function ConnectDeviceModal(props: DialogProps) {
  const { data: ports } = useQuery({
    queryKey: ["ports"],
    queryFn: async () =>
      uniqueBy(
        await invoke<SerialPort[]>("get_controllers"),
        (obj) => obj.port_type.UsbPort.serial_number,
      ),
    refetchInterval: 1000,
  });

  /*const { mutate: connect } = useMutation({
    mutationFn: async (port: string) => {
      await store?.set("controller_bus", port);
      await invoke("restart");
    },
  });*/

  return (
    <Dialog {...props}>
      <DialogContent>
        <div className="p-4">
          <DialogHeader>
            <DialogTitle>Connect controller</DialogTitle>

            <DialogDescription>
              Select a controller from the list below to connect. If none
              appear, ensure your device is powered on and connected. The list
              updates automatically when controllers are detected.
            </DialogDescription>

            <div className="h-[430px] space-y-4 overflow-auto pt-8">
              {ports?.length === 0 ? (
                <div className="flex w-full h-full items-center justify-center">
                  <div className="text-sm text-muted-foreground">
                    No controllers found
                  </div>
                </div>
              ) : (
                ports?.map((port) => (
                  <div className="flex justify-between" key={port.port_name}>
                    <div>
                      {port.port_type.UsbPort.manufacturer ?? port.port_name}
                      <Badge variant="tag" className="ml-2">
                        {port.port_type.UsbPort.serial_number}
                      </Badge>
                    </div>
                    <Button
                      size="sm"
                      variant="outline"
                      // onClick={() => connect(port.port_name)}
                    >
                      Connect
                    </Button>
                  </div>
                ))
              )}
            </div>
          </DialogHeader>
        </div>
      </DialogContent>
    </Dialog>
  );
}
