import AdminLayout from "@/components/layout/AdminLayout";

export default function AdminDashboard() {
  return (
    <AdminLayout>
      <div className="space-y-6">
        <div>
          <h1 className="text-3xl font-bold">Dashboard</h1>
          <p className="text-lg text-muted-foreground mt-2">
            Welcome to the ESPx administrative interface.
          </p>
        </div>

        {/* Quick Stats Section */}
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          <div className="rounded-lg border bg-card p-6">
            <h3 className="text-sm font-medium text-muted-foreground">Total Users</h3>
            <div className="mt-2 text-2xl font-bold">0</div>
          </div>
          <div className="rounded-lg border bg-card p-6">
            <h3 className="text-sm font-medium text-muted-foreground">Active Sessions</h3>
            <div className="mt-2 text-2xl font-bold">0</div>
          </div>
          <div className="rounded-lg border bg-card p-6">
            <h3 className="text-sm font-medium text-muted-foreground">Total Requests</h3>
            <div className="mt-2 text-2xl font-bold">0</div>
          </div>
          <div className="rounded-lg border bg-card p-6">
            <h3 className="text-sm font-medium text-muted-foreground">System Status</h3>
            <div className="mt-2 text-2xl font-bold">Active</div>
          </div>
        </div>
      </div>
    </AdminLayout>
  );
}
