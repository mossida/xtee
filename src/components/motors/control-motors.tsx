"use client";

import { motorModeAtom } from "@/state";
import * as TabsPrimitive from "@radix-ui/react-tabs";
import { useAtom } from "jotai";
import { useState } from "react";
import { ModeSelector } from "../mode-selector";
import { Card } from "../ui/card";
import { ManualMode } from "./modes/manual";
import { ServingMode } from "./modes/serving";
import { TwistingMode } from "./modes/twisting";

export function ControlMotors() {
  const [mode, setMode] = useAtom(motorModeAtom);

  return (
    <Card className="flex flex-col grow">
      <TabsPrimitive.Root className="p-6 space-y-6" value={mode}>
        <TabsPrimitive.List className="flex flex-row justify-between">
          <ModeSelector
            value={mode}
            onChange={(mode) => setMode(mode)}
            modes={["twisting", "serving", "manual"] as const}
          />
        </TabsPrimitive.List>
        <TabsPrimitive.Content value="twisting">
          <TwistingMode />
        </TabsPrimitive.Content>
        <TabsPrimitive.Content value="serving">
          <ServingMode />
        </TabsPrimitive.Content>
        <TabsPrimitive.Content value="manual">
          <ManualMode />
        </TabsPrimitive.Content>
      </TabsPrimitive.Root>
    </Card>
  );
}
