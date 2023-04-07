import { useAuth } from "@/components/hooks";
import { useRouter } from "next/router";
import { useEffect } from "react";

function Dashboard() {
  const router = useRouter();
  const { user } = useAuth();
  useEffect(() => {
    if (!user) {
      router.push("/login");
    }
  }, []);

  if (!user) {
    return <></>;
  }

  return <div></div>;
}

export default Dashboard;
