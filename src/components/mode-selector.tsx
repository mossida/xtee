"use client";

import { ChevronLeftIcon, ChevronRightIcon } from "@radix-ui/react-icons";
import { useCallback, useState } from "react";
import { Button } from "./ui/button";

export function ModeSelector<M extends ReadonlyArray<string>>({
  value,
  modes,
  onChange,
}: {
  value: M[number];
  modes: M;
  onChange: (mode: M[number]) => void;
}) {
  const nextMode = useCallback(() => {
    const index = modes.indexOf(value);
    onChange(modes[index + 1] ?? (modes[0] as M[number]));
  }, [value, onChange, modes]);

  const prevMode = useCallback(() => {
    const index = modes.indexOf(value);
    onChange(modes[index - 1] ?? (modes[modes.length - 1] as M[number]));
  }, [value, onChange, modes]);

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
