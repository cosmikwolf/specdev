// Case: the single-file configuration.
// Certifies the v5 shape the block is built on: one NextAuth() call
// whose destructured exports are what every consumer imports.
import NextAuth from "next-auth";
import GitHub from "next-auth/providers/github";

export const { handlers, auth, signIn, signOut } = NextAuth({
  providers: [GitHub],
});
