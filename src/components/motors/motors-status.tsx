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
      <div className="w-full flex justify-between items-center mb-2">
        <h4 className="text-base">Status</h4>
        <TabsList>
          <TabsTrigger value="1">Motor 1</TabsTrigger>
          <TabsTrigger value="2">Motor 2</TabsTrigger>
        </TabsList>
      </div>
      <TabsContent value="1">
        <MotorStatus motor={1} />
      </TabsContent>
      <TabsContent value="2">
        <MotorStatus motor={2} />
      </TabsContent>
    </Tabs>
  );
}
