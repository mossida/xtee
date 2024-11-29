"use client";

import { ConnectDeviceModal } from "@/components/modals/connect-device-modal";
import { storeQueryOptions } from "@/components/prefetch-provider";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { useState } from "react";
import { useQuery } from "react-query";

export default function GeneralSettings() {
  const [isOpen, setIsOpen] = useState(false);

  const { data: store } = useQuery(storeQueryOptions);
  const { data: controller } = useQuery({
    queryKey: ["controller"],
    queryFn: () => store?.get<string>("controller_bus"),
  });

  return (
    <div className="grid grid-cols-2 gap-4">
      <ConnectDeviceModal open={isOpen} onOpenChange={setIsOpen} />
      <Card>
        <CardHeader>
          <CardTitle>Controllers</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableCaption>Currently connected controllers</TableCaption>
            <TableHeader>
              <TableRow>
                <TableHead>Group</TableHead>
                <TableHead>Slave</TableHead>
                <TableHead>Baud rate</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow>
                <TableCell>
                  <Badge>default</Badge>
                </TableCell>
                <TableCell>{controller}</TableCell>
                <TableCell>115200</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </CardContent>
        <CardFooter className="flex justify-end gap-4">
          <Button onClick={() => setIsOpen(true)}>Change controller</Button>
          <Button variant="destructive" className="hover:bg-destructive">
            Restart
          </Button>
        </CardFooter>
      </Card>
      <Card />
    </div>
  );
}
