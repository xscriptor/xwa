import ReportDashboard from "@/components/ReportDashboard";
import { isReportTabKey } from "@/lib/navigation";
import { notFound } from "next/navigation";

export default async function ReportSectionPage(props: {
  params: Promise<{ id: string; section: string }>;
}) {
  const params = await props.params;

  if (!isReportTabKey(params.section)) {
    notFound();
  }

  return <ReportDashboard scanId={params.id} activeTab={params.section} />;
}
