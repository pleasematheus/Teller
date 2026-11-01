import { authedInvoke } from "@/lib/api";

export async function getInitialScreen(): Promise<"login" | "setup"> {
  const error = await authedInvoke<string | null>("get_startup_error");
  if (error) {
    throw new Error(error);
  }
  const hasUsers = await authedInvoke<boolean>("check_setup");
  return hasUsers ? "login" : "setup";
}
