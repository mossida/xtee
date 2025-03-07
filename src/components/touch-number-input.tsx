"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useLongPress } from "@/hooks/use-long-press";
import { cn } from "@/lib/utils";
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
    threshold: 1000,
    onStart: () => {
      timerRef.current = incrementInterval();
    },
    onEnd: stopTimer,
  });

  const decrementPress = useLongPress({
    threshold: 1000,
    onStart: () => {
      timerRef.current = decrementInterval();
    },
    onEnd: stopTimer,
  });

  return (
    <div className={cn("flex items-center space-x-2", className)}>
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
          className="h-14 text-center text-2xl"
        />
        {suffix && (
          <span className="pl-10 absolute right-3 top-1/2 -translate-y-1/2 text-secondary-foreground">
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
  );
}
