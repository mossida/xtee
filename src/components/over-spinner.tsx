import { cn } from "@/lib/utils";
import { Spinner } from "./ui/spinner";

export const OverSpinner = ({
  children,
  isLoading,
}: {
  children: React.ReactNode;
  isLoading: boolean;
}) => {
  return (
    <div className="grid grid-cols-1 grid-rows-1">
      <div
        className={cn(
          "col-span-1 row-span-1 w-full h-full flex items-center justify-center [grid-area:1/1] transition-opacity duration-300",
          isLoading ? "" : "opacity-0",
        )}
      >
        <Spinner size={32} />
      </div>
      <div
        className={cn(
          "col-span-1 row-span-1 [grid-area:1/1] transition-opacity duration-300 z-10",
          isLoading ? "opacity-50" : "",
        )}
      >
        {children}
      </div>
    </div>
  );
};
