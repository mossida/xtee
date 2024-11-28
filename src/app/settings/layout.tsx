import { SecondaryMenu } from "@/components/secondary-menu";
import { Separator } from "@/components/ui/separator";

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div>
      <h1 className="text-2xl font-medium border-b pb-4">Settings</h1>
      <SecondaryMenu
        className="mb-6"
        items={[
          {
            label: "General",
            path: "/settings/general",
          },
          {
            label: "Actuator",
            path: "/settings/actuator",
          },
        ]}
      />
      {children}
    </div>
  );
}
