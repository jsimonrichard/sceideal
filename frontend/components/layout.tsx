import { useAuth } from "./hooks";

export function TopBar() {
  const { user } = useAuth();
}
