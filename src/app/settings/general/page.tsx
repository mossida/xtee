"use client";

import { ControllersTable } from "@/components/controllers-table";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export default function GeneralSettings() {
  return (
    <div className="grid grid-cols-2 gap-4">
      <Card className="col-span-2">
        <CardHeader>
          <CardTitle>Controllers</CardTitle>
        </CardHeader>
        <CardContent>
          <ControllersTable />
        </CardContent>
        <CardFooter className="flex justify-end gap-4">
          <Button variant="destructive" className="hover:bg-destructive">
            Restart
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
