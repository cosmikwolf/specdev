// Case: a consumer module.
// Certifies that a module carrying `[prisma].client` reaches the
// singleton by import and that the model delegates are typed.
import { prisma } from "./singleton";

export async function listEvents() {
  return prisma.event.findMany({ orderBy: { startsAt: "asc" } });
}

export async function findEvent(id: string) {
  return prisma.event.findUnique({ where: { id } });
}
