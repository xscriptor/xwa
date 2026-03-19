import ReportDashboard from "@/components/ReportDashboard";

export default async function ReportPage(props: { params: Promise<{ id: string }> }) {
  const params = await props.params;
  return <ReportDashboard scanId={params.id} activeTab="overview" />;
}
