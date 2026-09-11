// Case: error mapping.
// Certifies that `PrismaClientKnownRequestError` is reachable from the
// generated client's `Prisma` namespace and that `.code` narrows in a
// type-safe way — the contract the prose's code table rests on.
import { prisma } from "./singleton";
import { Prisma } from "./generated/prisma/client";

export type CreateResult = { ok: true } | { ok: false; error: string };

export async function createUser(email: string): Promise<CreateResult> {
  try {
    await prisma.user.create({ data: { email } });
    return { ok: true };
  } catch (e) {
    if (e instanceof Prisma.PrismaClientKnownRequestError) {
      if (e.code === "P2002") return { ok: false, error: "that email is taken" };
      if (e.code === "P2025") return { ok: false, error: "not found" };
    }
    throw e;
  }
}

/// `P2034` is the one code the docs name as retryable. Retry the whole
/// `$transaction` call, never the inner query.
export function isRetryable(e: unknown): boolean {
  return e instanceof Prisma.PrismaClientKnownRequestError && e.code === "P2034";
}
