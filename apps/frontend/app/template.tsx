import Navbar from "@/components/Navbar";
import { getAuthStatus } from "@/app/actions/getAuthStatus";
import { AuthProvider } from "@/context/auth-context";

export default async function Template({
  children,
}: {
  children: React.ReactNode;
}) {
  const { email: userEmail } = await getAuthStatus();
  return (
    <div>
      <AuthProvider>
        <header>
      <Navbar userEmail={userEmail} />
        </header>
        {children}
      </AuthProvider>
    </div>
  );
}
