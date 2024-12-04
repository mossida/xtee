import { Badge } from "./ui/badge";
import { Separator } from "./ui/separator";

export function MotorStatus({ motor }: { motor: 1 | 2 }) {
  return (
    <div className="flex flex-col gap-2">
      <Separator className="my-3" />
      <div className="space-y-4">
        <div className="flex justify-between items-center">
          <span className="text-sm">Axis</span>
          <Badge variant="tag">LOCKED</Badge>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm">Condition</span>
          <Badge
            variant="tag"
            className="bg-green-500 dark:bg-green-600 text-black text-xs"
          >
            RUNNING
          </Badge>
        </div>
      </div>
    </div>
  );
}
