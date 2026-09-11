// Case: the derived route file.
// Certifies that `handlers` destructures into the GET and POST exports a
// Next.js route file must provide. Two lines, and they are the whole
// contents of app/api/auth/[...nextauth]/route.ts.
import { handlers } from "./config";

export const { GET, POST } = handlers;
