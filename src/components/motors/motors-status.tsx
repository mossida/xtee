import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { MotorStatus } from "./motor-status";

export function MotorsStatus() {
  return (
    <Tabs defaultValue="motor-1">
      <div className="w-full flex justify-between items-center mb-2">
        <h4 className="text-base">Status</h4>
        <TabsList>
          <TabsTrigger value="motor-1">Motor 1</TabsTrigger>
          <TabsTrigger value="motor-2">Motor 2</TabsTrigger>
        </TabsList>
      </div>
      <TabsContent value="motor-1">
        <MotorStatus motor={1} />
      </TabsContent>
      <TabsContent value="motor-2">
        <MotorStatus motor={2} />
      </TabsContent>
    </Tabs>
  );
}
