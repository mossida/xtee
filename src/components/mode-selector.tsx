"use client";

import { ChevronLeftIcon, ChevronRightIcon } from "@radix-ui/react-icons";
import { useCallback, useState } from "react";
import { Button } from "./ui/button";

export const MODES = ["twisting", "serving", "manual"] as const;

export type Mode = (typeof MODES)[number];

export function ModeSelector({
  value,
  onChange,
}: {
  value: Mode;
  onChange: (mode: Mode) => void;
}) {
  const nextMode = useCallback(() => {
    const index = MODES.indexOf(value);
    onChange(MODES[index + 1] ?? MODES[0]);
  }, [value, onChange]);

  const prevMode = useCallback(() => {
    const index = MODES.indexOf(value);
    onChange(MODES[index - 1] ?? (MODES[MODES.length - 1] as Mode));
  }, [value, onChange]);

  return (
    <>
      <Button size="icon" variant="secondary" onClick={prevMode}>
        <ChevronLeftIcon className="w-4 h-4" />
      </Button>
      <span className="text-lg font-mono">{value.toUpperCase()}</span>
      <Button size="icon" variant="secondary" onClick={nextMode}>
        <ChevronRightIcon className="w-4 h-4" />
      </Button>
    </>
  );
}
