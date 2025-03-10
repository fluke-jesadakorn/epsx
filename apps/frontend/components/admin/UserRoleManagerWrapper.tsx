import { listUsers } from "@/app/actions/auth-server";
import UserRoleManager from "./UserRoleManager";

export default async function UserRoleManagerWrapper() {
  const users = await listUsers();
  return <UserRoleManager users={users} />;
}
