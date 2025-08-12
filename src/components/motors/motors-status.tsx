import { motorStatusViewAtom } from "@/state";
import { useAtom } from "jotai";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { MotorStatus } from "./motor-status";

export function MotorsStatus() {
  const [view, setView] = useAtom(motorStatusViewAtom);

  return (
    <Tabs
      defaultValue="1"
      value={view}
      onValueChange={(value) => setView(value as "1" | "2")}
    >
      <h4 className="text-base mb-4">Status</h4>
      <TabsList className="w-full">
        <TabsTrigger className="w-full" value="1">
          Motor 1
        </TabsTrigger>
        <TabsTrigger className="w-full" value="2">
          Motor 2
        </TabsTrigger>
      </TabsList>
      <TabsContent value="1">
        <MotorStatus motor={1} />
      </TabsContent>
      <TabsContent value="2">
        <MotorStatus motor={2} />
      </TabsContent>
    </Tabs>
  );
}
