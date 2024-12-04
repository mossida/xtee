import { cn } from "@/lib/cn";

export function LoadVisualizer({
  max,
  current,
}: {
  current: number;
  max: number;
}) {
  const getColor = (index: number) => {
    const percentage = current / max;
    const barPercentage = (index + 1) / 12;

    if (barPercentage <= percentage) {
      if (percentage <= 0.4) return "bg-green-500";
      if (percentage <= 0.8) return "bg-yellow-500";
      return "bg-red-500";
    }

    return "bg-primary/20";
  };

  return (
    <div className="flex items-end gap-[8px]">
      {Array.from({ length: 12 }).map((_, i) => (
        <div key={`load-bar-${i}`} className="relative">
          <div
            className={cn(
              "w-[8px] h-[50px] transition-colors duration-200",
              getColor(i),
            )}
          />
        </div>
      ))}
    </div>
  );
}
