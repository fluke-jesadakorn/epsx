import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { UserRole } from "@/constants/roles";
import { UserRoleManager } from "@/components/admin/UserRoleManager";

async function getUsers() {
  try {
    const cookieStore = await cookies();
    const session = cookieStore.get("__session");

    if (!session) {
      redirect("/login");
    }

    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/auth/users`,
      {
        headers: {
          Cookie: `__session=${session.value}`,
        },
        credentials: "include",
        cache: "no-store", // Disable caching for admin data
      }
    );

    if (!response.ok) {
      if (response.status === 401 || response.status === 403) {
        redirect("/unauthorized");
      }
      throw new Error("Failed to fetch users");
    }

    const data = await response.json();
    return data.users.map((user: any) => ({
      userId: user.userId,
      email: user.email,
      role: user.role as UserRole,
    }));
  } catch (error) {
    console.error("Error fetching users:", error);
    throw error;
  }
}

export default async function AdminRolesPage() {
  const users = await getUsers();

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold mb-6">User Role Management</h1>
      <UserRoleManager users={users} />
    </div>
  );
}
