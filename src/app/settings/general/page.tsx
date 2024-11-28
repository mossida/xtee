"use client";

import { ConnectDeviceModal } from "@/components/modals/connect-device-modal";
import { storeQueryOptions } from "@/components/prefetch-provider";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
          <CardTitle>Controller</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            <div className="space-y-2">
              <div className="text-sm">Current controller</div>
              <div className="font-medium">
                {controller ? controller : "Not connected"}
              </div>
              <p className="text-sm text-muted-foreground">
                Connect a controller to start using the application. The
                controller manages all the components and provides real-time
                feedback.
              </p>
            </div>

            <div className="space-y-2">
              <div className="text-sm">Baud rate</div>
              <Select>
                <SelectTrigger className="w-[180px]">
                  <SelectValue placeholder="Baud rate" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="9600">9600</SelectItem>
                  <SelectItem value="19200">19200</SelectItem>
                  <SelectItem value="38400">38400</SelectItem>
                  <SelectItem value="57600">57600</SelectItem>
                  <SelectItem value="115200">115200</SelectItem>
                </SelectContent>
              </Select>
              <p className="text-sm text-muted-foreground">
                The baud rate determines the speed of communication between the
                computer and controller. Higher rates allow for faster data
                transfer but may be less stable on some devices.
              </p>
            </div>
          </div>
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
