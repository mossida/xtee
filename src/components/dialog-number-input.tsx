"use client";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTrigger } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { VisuallyHidden } from "@/components/ui/visually-hidden";
import { DialogTitle } from "@radix-ui/react-dialog";
import { useCallback, useEffect, useMemo, useState } from "react";
import * as R from "remeda";
import { z } from "zod";

const KEYPAD_NUMBERS = ["7", "8", "9", "4", "5", "6", "1", "2", "3"] as const;

type KeypadKey =
  | "0"
  | (typeof KEYPAD_NUMBERS)[number]
  | "."
  | "backspace"
  | "clear"
  | "negate";

interface DialogNumberInputProps {
  ref?: React.Ref<HTMLInputElement>;
  name: string;
  value: number;
  disabled?: boolean;
  onChange: (value: number) => void;
  onBlur: () => void;
  min?: number;
  max?: number;
  allowFloat?: boolean;
  allowNegative?: boolean;
}

const createNumberSchema = ({
  min,
  max,
  allowNegative,
}: Pick<DialogNumberInputProps, "min" | "max" | "allowNegative">) => {
  let schema = z.number({
    required_error: "Please enter a value",
    invalid_type_error: "Please enter a valid number",
  });

  if (!allowNegative) {
    schema = schema.nonnegative({ message: "Negative values are not allowed" });
  }

  if (min !== undefined) {
    schema = schema.min(min, { message: `Value must be at least ${min}` });
  }

  if (max !== undefined) {
    schema = schema.max(max, { message: `Value must be at most ${max}` });
  }

  return schema;
};

const formatDisplayValue = (value: string): string => {
  return R.pipe(
    value,
    (v) => v || "0",
    (v) => (v === "-" ? "0" : v),
    (v) => {
      const num = Number.parseFloat(v);
      return Number.isNaN(num) ? "0" : v;
    },
  );
};

const validateNumber = (
  value: string,
  {
    min,
    max,
    allowFloat,
    allowNegative,
  }: Pick<
    DialogNumberInputProps,
    "min" | "max" | "allowFloat" | "allowNegative"
  >,
): string => {
  if (!value || value === "-") return value;
  if (allowFloat && (value === "." || value === "-.")) return `0${value}`;

  return R.pipe(
    value,
    (v) => (allowFloat ? v : v.replace(/\./g, "")),
    (v) => (allowNegative ? v : v.replace(/^-/, "")),
    (v) => {
      if (allowFloat && (v.endsWith(".") || v.match(/\.\d*$/))) return v;

      const num = Number.parseFloat(v);
      if (Number.isNaN(num)) return v;

      return R.pipe(
        num,
        (n) => (min !== undefined ? Math.max(n, min) : n),
        (n) => (max !== undefined ? Math.min(n, max) : n),
        (n) => n.toString(),
      );
    },
  );
};

export function DialogNumberInput({
  ref,
  name,
  value,
  onChange,
  onBlur,
  disabled,
  min,
  max,
  allowFloat = true,
  allowNegative = true,
}: DialogNumberInputProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [tempValue, setTempValue] = useState(value.toString());
  const [error, setError] = useState<string | null>(null);

  const schema = createNumberSchema({ min, max, allowNegative });

  useEffect(() => {
    setTempValue(value.toString());
    setError(null);
  }, [value]);

  const validateAndSetValue = useCallback(
    (newValue: string) => {
      const validatedValue = validateNumber(newValue, {
        min,
        max,
        allowFloat,
        allowNegative,
      });

      setTempValue(validatedValue);
      setError(null);
    },
    [allowFloat, allowNegative, min, max],
  );

  const handleKeyPress = useCallback(
    (key: KeypadKey) => {
      const keyActions: Record<KeypadKey, () => void> = {
        backspace: () => validateAndSetValue(tempValue.slice(0, -1)),
        clear: () => validateAndSetValue(""),
        negate: () =>
          allowNegative &&
          validateAndSetValue(
            tempValue.startsWith("-") ? tempValue.slice(1) : `-${tempValue}`,
          ),
        ".": () => {
          if (allowFloat && !tempValue.includes(".")) {
            validateAndSetValue(tempValue === "" ? "0." : `${tempValue}.`);
          }
        },
        "0": () => validateAndSetValue(`${tempValue}${key}`),
        "1": () => validateAndSetValue(`${tempValue}${key}`),
        "2": () => validateAndSetValue(`${tempValue}${key}`),
        "3": () => validateAndSetValue(`${tempValue}${key}`),
        "4": () => validateAndSetValue(`${tempValue}${key}`),
        "5": () => validateAndSetValue(`${tempValue}${key}`),
        "6": () => validateAndSetValue(`${tempValue}${key}`),
        "7": () => validateAndSetValue(`${tempValue}${key}`),
        "8": () => validateAndSetValue(`${tempValue}${key}`),
        "9": () => validateAndSetValue(`${tempValue}${key}`),
      };

      keyActions[key]?.();
    },
    [tempValue, allowFloat, allowNegative, validateAndSetValue],
  );

  const handleEnter = useCallback(() => {
    const numValue = Number.parseFloat(tempValue || "0");
    const result = schema.safeParse(numValue);

    if (!result.success) {
      setError(result.error.errors[0]?.message || "Invalid value");
      return;
    }

    onChange(result.data);
    setIsOpen(false);
    setError(null);
  }, [tempValue, onChange, schema]);

  const displayValue = formatDisplayValue(tempValue);

  return (
    <div className="space-y-2 grow">
      <Dialog open={isOpen} onOpenChange={setIsOpen} modal={false}>
        <DialogTrigger asChild>
          <Input
            ref={ref}
            type="text"
            id={name}
            name={name}
            value={value.toString()}
            disabled={disabled}
            onBlur={onBlur}
            readOnly
            className="cursor-pointer font-mono"
            aria-label={`Open numeric keypad for ${name}`}
          />
        </DialogTrigger>
        <VisuallyHidden>
          <DialogTitle>{name}</DialogTitle>
        </VisuallyHidden>

        <DialogContent className="sm:max-w-[425px] p-7!">
          <div className="grid gap-4">
            <div className="space-y-1">
              <div
                className="text-2xl font-bold text-center font-mono"
                aria-live="polite"
              >
                {displayValue}
              </div>
              {error && (
                <p
                  className="text-sm text-center text-destructive"
                  role="alert"
                >
                  {error}
                </p>
              )}
            </div>
            <div className="grid grid-cols-3 gap-2">
              {KEYPAD_NUMBERS.map((num) => (
                <Button
                  size="lg"
                  key={num}
                  onClick={() => handleKeyPress(num)}
                  disabled={disabled}
                >
                  {num}
                </Button>
              ))}
              <Button onClick={() => handleKeyPress("0")} disabled={disabled}>
                0
              </Button>
              <Button
                onClick={() => handleKeyPress(".")}
                disabled={!allowFloat || disabled}
              >
                .
              </Button>
              <Button
                onClick={() => handleKeyPress("negate")}
                disabled={!allowNegative || disabled}
              >
                +/-
              </Button>
            </div>
            <div className="grid grid-cols-2 gap-2">
              <Button
                variant="outline"
                onClick={() => handleKeyPress("clear")}
                disabled={disabled}
              >
                Clear
              </Button>
              <Button
                variant="outline"
                onClick={() => handleKeyPress("backspace")}
                disabled={disabled}
              >
                ⌫
              </Button>
            </div>
            <Button
              onClick={handleEnter}
              className="w-full"
              size="lg"
              disabled={disabled}
            >
              Enter
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
