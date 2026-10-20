import { DocsNav } from "@/components/docs-nav";
export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="page-shell grid min-h-[75vh] lg:grid-cols-[230px_minmax(0,1fr)]">
      <DocsNav />
      {children}
    </div>
  );
}
