import { Lock, RotateCcw } from "lucide-react";
import { Toggle } from "../ui/toggle";

export function MotorLock({
  current,
  onPressedChange,
}: {
  current: boolean;
  onPressedChange: (pressed: boolean) => void;
}) {
  return (
    <Toggle
      pressed={current}
      className={`
    border transition-all duration-200 min-w-[80px] h-10 rounded-none
    ${
      current
        ? "data-[state=on]:bg-yellow-500 data-[state=on]:text-black data-[state=on]:border-yellow-600"
        : "data-[state=off]:bg-muted data-[state=off]:text-muted-foreground data-[state=off]:border-border"
    }
  `}
      onPressedChange={onPressedChange}
    >
      {current ? (
        <>
          <Lock className="w-4 h-4" />
          <span className="text-xs">LOCKED</span>
        </>
      ) : (
        <>
          <RotateCcw className="w-4 h-4" />
          <span className="text-xs">FREE</span>
        </>
      )}
    </Toggle>
  );
}
