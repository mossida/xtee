"use client";

import { useEvent, waitEvent } from "@/hooks/use-event";
import { api } from "@/lib/client";
import { store } from "@/lib/store";
import type { Controller, ControllerGroup, Port } from "@/types/bindings";
import { useMutation } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { uniqueBy } from "remeda";
import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { ComboboxDropdown } from "./ui/combobox";
import { Spinner } from "./ui/spinner";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./ui/table";
import { toast } from "sonner";

function useControllerStatus() {
  const event = useEvent("controller-status");

  useEffect(() => {
    if (event) {
      if (event.status.type === "connected") {
        toast.success("Controller connected");
      } else if (event.status.type === "disconnected") {
        toast.info("Controller disconnected");
      } else if (event.status.type === "failed") {
        toast.warning("Controller failed", {
          description: event.status.data.reason,
        });
      }
    }
  }, [event]);
}

export function ControllersTable() {
  useControllerStatus();

  const queries = api.useQueries([
    ["master/ports", void 0, { refetchInterval: 1000 }],
    ["master/groups", void 0, {}],
    ["master/controllers", void 0, { refetchInterval: 1000 }],
  ]);

  const { mutateAsync: save } = store.useMutation();

  const [ports, groups, controllers] = queries.map((query) => query.data) as [
    Port[],
    ControllerGroup[],
    Controller[],
  ];

  const groupItems =
    groups?.map((group) => ({ id: group, label: group })) ?? [];
  const groupItemsById = new Map(groupItems.map((item) => [item.id, item]));

  const [groupSelections, setGroupSelections] = useState<
    Record<string, ControllerGroup>
  >({});

  const uniquePorts = uniqueBy(ports ?? [], (p) => p.serial_number);
  const controllersByPort = new Map(
    controllers?.map((c) => [c.serial_port, c]) ?? [],
  );

  const { mutateAsync: spawn } = api.useMutation("master/spawn");
  const { mutateAsync: kill } = api.useMutation("master/kill");
  const {
    mutate: connect,
    isPending,
    variables,
  } = useMutation({
    mutationFn: async (port: string) => {
      const id = crypto.randomUUID();

      const controller = {
        serial_port: port,
        group: groupSelections[port] as ControllerGroup,
        baud_rate: 115200,
        id,
      };

      await spawn(controller);

      const result = Promise.race([
        new Promise((resolve) => setTimeout(resolve, 3000)),
        waitEvent(
          "controller-status",
          ({ data: { controller, status } }) =>
            controller.id === id && status.type === "connected",
        ),
      ]);

      if (result === null) {
        throw new Error("Failed to connect to controller");
      }

      await save([["controllers.spawn", [...(controllers ?? []), controller]]]);
    },
  });

  const { mutate: disconnect } = useMutation({
    mutationFn: async (id: string) => {
      await kill(id);

      const result = Promise.race([
        new Promise((resolve) => setTimeout(resolve, 3000)),
        waitEvent(
          "controller-status",
          ({ data: { controller, status } }) =>
            controller.id === id && status.type === "disconnected",
        ),
      ]);

      if (result === null) {
        throw new Error("Failed to disconnect from controller");
      }
    },
  });

  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead className="w-[100px]">Name</TableHead>
          <TableHead>Manufacturer</TableHead>
          <TableHead className="w-fit">Serial Number</TableHead>
          <TableHead>Group</TableHead>
          <TableHead className="text-center">Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {uniquePorts.map((port) => {
          const isConnected = controllersByPort.has(port.name);

          const controller = controllersByPort.get(port.name);
          const variant = isConnected ? "destructive" : "default";
          const buttonText = isConnected ? "Disconnect" : "Connect";
          const action = isConnected
            ? () => disconnect(controller?.id ?? "")
            : connect;

          const group =
            isConnected && controller?.group
              ? controller.group
              : groupSelections[port.name];

          return (
            <TableRow key={port.name}>
              <TableCell className="font-medium">{port.name}</TableCell>
              <TableCell>{port.manufacturer}</TableCell>
              <TableCell>
                <Badge variant="tag">{port.serial_number ?? "N/A"}</Badge>
              </TableCell>
              <TableCell>
                <ComboboxDropdown
                  hasSearch={false}
                  popoverProps={{ className: "!animate-none" }}
                  disabled={isConnected}
                  items={groupItems.map((item) => ({
                    ...item,
                    disabled: isConnected && group === item.id,
                  }))}
                  selectedItem={group ? groupItemsById.get(group) : undefined}
                  onSelect={({ id }) => {
                    setGroupSelections((prev) => ({
                      ...prev,
                      [port.name]: id,
                    }));
                  }}
                />
              </TableCell>
              <TableCell>
                <Button
                  className="w-full"
                  onClick={() => action(port.name)}
                  variant={variant}
                  disabled={
                    (!isConnected && !groupSelections[port.name]) || isPending
                  }
                >
                  {isPending && variables === port.name ? (
                    <Spinner />
                  ) : (
                    buttonText
                  )}
                </Button>
              </TableCell>
            </TableRow>
          );
        })}
      </TableBody>
    </Table>
  );
}
