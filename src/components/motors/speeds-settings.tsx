"use client";

import { cn } from "@/lib/utils";
import { speedToRpm } from "@/lib/constants";
import { type Store, store } from "@/lib/store";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { mapValues } from "remeda";
import { z } from "zod";
import { DialogNumberInput } from "../dialog-number-input";
import { OverSpinner } from "../over-spinner";
import { Button } from "../ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "../ui/card";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "../ui/form";
import { Spinner } from "../ui/spinner";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";

export const servingSpeeds = ["slow", "medium", "fast"] as const;
export const twistingSpeeds = ["slow", "fast"] as const;

type ServingSpeed = (typeof servingSpeeds)[number];
type TwistingSpeed = (typeof twistingSpeeds)[number];

const speedSchema = z.object({
  twisting: z.object(
    Object.fromEntries(
      twistingSpeeds.map((speed) => [speed, z.number().min(1).max(1000)]),
    ) as Record<TwistingSpeed, z.ZodNumber>,
  ),
  serving: z.object(
    Object.fromEntries(
      servingSpeeds.map((speed) => [speed, z.number().min(1).max(1000)]),
    ) as Record<ServingSpeed, z.ZodNumber>,
  ),
});

type SpeedSettings = z.infer<typeof speedSchema>;

export function SpeedsSettings() {
  "use no memo";

  const [activeTab, setActiveTab] = useState("twisting");

  const queries = store.useQueries(["motors.limits", "motors.speeds"]);

  const isFetching = queries.some((query) => query.isFetching);

  const [motorsLimits, motorsSpeeds] = queries.map((query) => query.data) as [
    Store["motors.limits"] | null,
    Store["motors.speeds"] | null,
  ];

  const { mutateAsync: save } = store.useMutation();

  const spp = motorsLimits?.stepsPerPulse ?? 800;
  const form = useForm<SpeedSettings>({
    resolver: zodResolver(speedSchema),
    values: motorsSpeeds ?? {
      twisting: { slow: 100, fast: 300 },
      serving: { slow: 50, medium: 100, fast: 150 },
    },
  });

  const { mutate, isPending } = useMutation({
    mutationFn: async (data: SpeedSettings) => {
      await save([["motors.speeds", data]]);
    },
  });

  return (
    <Card className="w-full flex flex-col">
      <CardHeader>
        <CardTitle>Speeds</CardTitle>
        <CardDescription>
          Configure speed settings for both twisting and serving modes.
        </CardDescription>
      </CardHeader>
      <CardContent className="grow">
        <OverSpinner isLoading={isFetching}>
          <Form {...form}>
            <Tabs value={activeTab} onValueChange={setActiveTab}>
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="twisting">Twisting Mode</TabsTrigger>
                <TabsTrigger value="serving">Serving Mode</TabsTrigger>
              </TabsList>
              <TabsContent value="twisting" className="space-y-4">
                <div
                  className={cn(
                    "grid gap-4 pt-4",
                    `grid-cols-${twistingSpeeds.length}`,
                  )}
                >
                  {twistingSpeeds.map((speed) => (
                    <FormField
                      key={speed}
                      control={form.control}
                      name={`twisting.${speed}`}
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel className="capitalize">{speed}</FormLabel>
                          <FormControl>
                            <div className="flex items-center space-x-2">
                              <DialogNumberInput
                                min={1}
                                max={speedToRpm(
                                  motorsLimits?.maxSpeed ?? 1,
                                  spp,
                                )}
                                allowFloat={false}
                                allowNegative={false}
                                {...field}
                              />
                              <span className="text-sm text-muted-foreground">
                                RPM
                              </span>
                            </div>
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                  ))}
                </div>
              </TabsContent>
              <TabsContent value="serving" className="space-y-4">
                <div
                  className={cn(
                    "grid gap-4 pt-4",
                    `grid-cols-${servingSpeeds.length}`,
                  )}
                >
                  {servingSpeeds.map((speed) => (
                    <FormField
                      key={speed}
                      control={form.control}
                      name={`serving.${speed}`}
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel className="capitalize">{speed}</FormLabel>
                          <FormControl>
                            <div className="flex items-center space-x-2">
                              <DialogNumberInput
                                min={1}
                                max={speedToRpm(
                                  motorsLimits?.maxSpeed ?? 1,
                                  spp,
                                )}
                                allowFloat={false}
                                allowNegative={false}
                                {...field}
                              />
                              <span className="text-sm text-muted-foreground">
                                RPM
                              </span>
                            </div>
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                  ))}
                </div>
              </TabsContent>
            </Tabs>
          </Form>
        </OverSpinner>
      </CardContent>
      <CardFooter className="flex justify-end">
        <Button
          size={"lg"}
          type="submit"
          disabled={isPending || !form.formState.isDirty}
          onClick={() => mutate(form.getValues())}
        >
          {isPending ? <Spinner size={16} /> : "Save"}
        </Button>
      </CardFooter>
    </Card>
  );
}
