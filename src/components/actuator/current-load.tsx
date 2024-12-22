"use client";

import { useEvent } from "@/hooks/use-event";
import { LoadVisualizer } from "../load-visualizer";

function formatBalancedNumber(num: number): string {
  if (num === 0) return "0.00";
  if (num < 10) return num.toFixed(3); // 0-9: x.xxx
  if (num < 100) return num.toFixed(3); // 10-99: xx.xxx
  return num.toFixed(2); // 100+: xxx.xx
}

export function CurrentLoad() {
  const weight = useEvent("weight");

  return (
    <div className="flex flex-col items-start w-full">
      <span className="text-sm text-muted-foreground">Current load (kg)</span>
      <span className="font-mono font-medium text-5xl mb-6 mt-3">
        {formatBalancedNumber(weight ?? 0)}
      </span>
      <LoadVisualizer current={weight ?? 0} max={200} />
    </div>
  );
}
