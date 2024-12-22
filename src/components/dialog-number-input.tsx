"use client";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogOverlay,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { DialogTitle } from "@radix-ui/react-dialog";
import { VisuallyHidden } from "@radix-ui/react-visually-hidden";
import { useCallback, useEffect, useRef, useState } from "react";

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

  useEffect(() => {
    setTempValue(value.toString());
  }, [value]);

  const validateAndSetValue = useCallback(
    (newValue: string) => {
      let validatedValue = newValue;

      if (!allowFloat) {
        validatedValue = validatedValue.replace(/\./g, "");
      }

      if (!allowNegative) {
        validatedValue = validatedValue.replace(/^-/, "");
      }

      const numValue = Number.parseFloat(validatedValue);

      if (!Number.isNaN(numValue)) {
        if (min !== undefined && numValue < min) {
          validatedValue = min.toString();
        }
        if (max !== undefined && numValue > max) {
          validatedValue = max.toString();
        }
      }

      setTempValue(validatedValue);
    },
    [allowFloat, allowNegative, min, max],
  );

  const handleKeyPress = useCallback(
    (key: string) => {
      if (key === "backspace") {
        validateAndSetValue(tempValue.slice(0, -1));
      } else if (key === "clear") {
        validateAndSetValue("");
      } else if (key === "negate" && allowNegative) {
        validateAndSetValue(
          tempValue.startsWith("-") ? tempValue.slice(1) : `-${tempValue}`,
        );
      } else if (key === "." && allowFloat && !tempValue.includes(".")) {
        validateAndSetValue(`${tempValue}${key}`);
      } else if (key !== "." && key !== "negate") {
        validateAndSetValue(`${tempValue}${key}`);
      }
    },
    [tempValue, allowFloat, allowNegative, validateAndSetValue],
  );

  const handleEnter = useCallback(() => {
    const numValue = Number.parseFloat(tempValue || "0");
    onChange(numValue);
    setIsOpen(false);
  }, [tempValue, onChange]);

  return (
    <div className="space-y-2 flex-grow">
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

        <DialogContent className="sm:max-w-[425px] !p-7">
          <div className="grid gap-4">
            <div
              className="text-2xl font-bold text-center font-mono"
              aria-live="polite"
            >
              {tempValue || "0"}
            </div>
            <div className="grid grid-cols-3 gap-2">
              {["7", "8", "9", "4", "5", "6", "1", "2", "3"].map((num) => (
                <Button
                  size={"lg"}
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
              size={"lg"}
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
