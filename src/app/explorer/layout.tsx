import { ExplorerNav } from "@/components/explorer-nav";
export default function ExplorerLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="page-shell grid min-h-[80vh] lg:grid-cols-[230px_minmax(0,1fr)]">
      <ExplorerNav />
      {children}
    </div>
  );
}
