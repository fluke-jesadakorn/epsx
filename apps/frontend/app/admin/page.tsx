import { fetchUserDetails } from "@/app/actions/admin-server";
import UserManagementSection from "@/components/admin/UserManagementSection";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export default async function AdminPage() {
  const users = await fetchUserDetails();

  return (
    <div className="container mx-auto py-8 space-y-8">
      <Card className="mb-8">
        <CardHeader>
          <CardTitle>Admin Dashboard</CardTitle>
          <CardDescription>
            Manage users, roles, and system permissions
          </CardDescription>
        </CardHeader>
      </Card>

      <div className="grid gap-8">
        <UserManagementSection users={users} />
      </div>
    </div>
  );
}
