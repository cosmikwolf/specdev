// Case: transactions.
// Certifies both forms in the prose — the interactive callback with its
// three options, and the array form — including that `tx` carries the
// same model delegates as the outer client.
import { prisma } from "./singleton";
import { Prisma } from "./generated/prisma/client";

export async function bookSeat(id: string, seat: string): Promise<void> {
  await prisma.$transaction(
    async (tx) => {
      const event = await tx.event.findUniqueOrThrow({ where: { id } });
      await tx.booking.create({ data: { eventId: event.id, seat } });
    },
    {
      maxWait: 5000,
      timeout: 10000,
      isolationLevel: Prisma.TransactionIsolationLevel.Serializable,
    },
  );
}

export async function seedTwoUsers(a: string, b: string): Promise<void> {
  await prisma.$transaction([
    prisma.user.create({ data: { email: a } }),
    prisma.user.create({ data: { email: b } }),
  ]);
}
