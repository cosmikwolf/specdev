// Case: reading the session.
// Certifies that `auth()` is awaitable and that the result narrows — the
// surface every consumer module uses.
import { auth } from "./config";

export async function currentUserEmail(): Promise<string | null> {
  const session = await auth();
  return session?.user?.email ?? null;
}
