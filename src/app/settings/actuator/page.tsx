"use client";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useForm } from "react-hook-form";
import { useMutation, useQuery } from "@tanstack/react-query";

export default function ActuatorSettings() {
  const { register, handleSubmit } = useForm({});

  return (
    <div className="grid grid-cols-2 gap-4">
      <Card>
        <CardHeader>
          <CardTitle>PID settings</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <span className="text-sm text-muted-foreground">
                Proportional
              </span>
              <Input type="number" step={0.01} {...register("proportional")} />
            </div>
            <div className="flex flex-col gap-2">
              <span className="text-sm text-muted-foreground">Integral</span>
              <Input type="number" step={0.01} {...register("integral")} />
            </div>
            <div className="flex flex-col gap-2">
              <span className="text-sm text-muted-foreground">Derivative</span>
              <Input type="number" step={0.01} {...register("derivative")} />
            </div>
          </div>
        </CardContent>
        <CardFooter className="flex justify-end gap-4">
          <Button>Save</Button>
        </CardFooter>
      </Card>
    </div>
  );
}
