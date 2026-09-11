// Case: the join product.
// Certifies that PrismaAdapter accepts the pinned PrismaClient and that
// the result is assignable to NextAuth's `adapter` option — the single
// claim this bridge exists to make.
import { PrismaAdapter } from "@auth/prisma-adapter";
import NextAuth from "next-auth";
import GitHub from "next-auth/providers/github";

import { prisma } from "./db";

export const { handlers, auth, signIn, signOut } = NextAuth({
  adapter: PrismaAdapter(prisma),
  session: { strategy: "jwt" },
  providers: [GitHub],
});
