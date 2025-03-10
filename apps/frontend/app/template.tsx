import Navbar from "@/components/Navbar";
import { getAuthStatus } from "@/app/actions/getAuthStatus";
import { AuthProvider } from "@/context/auth-context";

export default async function Template({
  children,
}: {
  children: React.ReactNode;
}) {
  const {
    isAuthenticated: isLoggedIn,
    email: userEmail,
    role,
  } = await getAuthStatus();
  const isAdmin = role === "admin";

  return (
    <div>
      <AuthProvider>
        <header>
      <Navbar
        isAdmin={isAdmin}
        userEmail={userEmail}
      />
        </header>
        {children}
      </AuthProvider>
    </div>
  );
}
