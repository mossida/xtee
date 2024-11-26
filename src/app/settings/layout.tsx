import { SecondaryMenu } from "@/components/secondary-menu";

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div>
      <h1>Settings</h1>
      <SecondaryMenu items={[]} />
      {children}
    </div>
  );
}
