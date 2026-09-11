// Case: the Edge-compatible split.
// Certifies that a bare config object types as NextAuthConfig on its own
// and can be spread into the full call — the shape `edge_split = true`
// selects. The edge-safe half must import nothing Node-only.
import type { NextAuthConfig } from "next-auth";
import NextAuth from "next-auth";
import GitHub from "next-auth/providers/github";

export const authConfig = {
  providers: [GitHub],
  pages: { signIn: "/signin" },
} satisfies NextAuthConfig;

export const { handlers, auth, signIn, signOut } = NextAuth({
  ...authConfig,
});
