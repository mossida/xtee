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
import { useZoom } from "@/hooks/use-zoom";

const ZOOM_LEVELS = [0.5, 0.75, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0];

export default function GeneralSettings() {
  const [zoom, setZoom] = useZoom();

  return (
    <div className="grid grid-cols-2 gap-4">
      <Card className="col-span-2">
        <CardHeader>
          <CardTitle>Interface zoom</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex flex-wrap gap-2">
              {ZOOM_LEVELS.map((level) => (
                <Button
                  key={level}
                  variant={zoom === level ? "default" : "outline"}
                  size="sm"
                  onClick={() => setZoom(level)}
                >
                  {(level * 100).toFixed(0)}%
                </Button>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      <Card className="col-span-2">
        <CardHeader>
          <CardTitle>Controllers</CardTitle>
        </CardHeader>
        <CardContent>
          <ControllersTable />
        </CardContent>
      </Card>
    </div>
  );
}
