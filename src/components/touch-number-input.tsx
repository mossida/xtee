"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { useLongPress } from "@/hooks/use-long-press";
import { cn } from "@/lib/cn";
import { Minus, Plus } from "lucide-react";
import React, { useRef } from "react";

interface TouchNumberInputProps {
  min?: number;
  max?: number;
  step?: number;
  value?: number;
  suffix?: string;
  onChange?: (step: number) => void;
  className?: string;
}

export function TouchNumberInput({
  min = 0,
  max = 100,
  step = 1,
  value = 0,
  suffix,
  onChange,
  className,
}: TouchNumberInputProps) {
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  const handleChange = (step: number) => {
    onChange?.(step);
  };

  const increment = () => handleChange(step);
  const decrement = () => handleChange(-step);

  const incrementInterval = () => setInterval(() => increment(), 30);
  const decrementInterval = () => setInterval(() => decrement(), 30);

  const stopTimer = () => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  };

  const incrementPress = useLongPress({
    onStart: () => {
      timerRef.current = incrementInterval();
    },
    onEnd: stopTimer,
  });

  const decrementPress = useLongPress({
    onStart: () => {
      timerRef.current = decrementInterval();
    },
    onEnd: stopTimer,
  });

  return (
    <div className={cn("w-full max-w-sm space-y-4", className)}>
      <div className="flex items-center space-x-2">
        <Button
          ref={decrementPress}
          variant="outline"
          size="icon"
          onClick={() => decrement()}
          disabled={value <= min}
          className="h-14 w-14 text-2xl"
        >
          <Minus className="h-6 w-6" />
        </Button>
        <div className="relative flex-1">
          <Input
            type="number"
            value={value}
            onChange={(e) => handleChange(Number(e.target.value))}
            min={min}
            max={max}
            step={step}
            readOnly
            className="h-14 text-center text-2xl pr-10"
          />
          {suffix && (
            <span className="absolute right-3 top-1/2 -translate-y-1/2 text-secondary-foreground">
              {suffix}
            </span>
          )}
        </div>
        <Button
          ref={incrementPress}
          variant="outline"
          size="icon"
          onClick={() => increment()}
          disabled={value >= max}
          className="h-14 w-14 text-2xl"
        >
          <Plus className="h-6 w-6" />
        </Button>
      </div>
      <Slider
        value={[value]}
        min={min}
        max={max}
        step={step}
        onValueChange={([newValue]) => handleChange((newValue ?? 0) - value)}
      />
    </div>
  );
}
