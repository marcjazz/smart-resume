import { z } from "zod";
import { createTRPCRouter, protectedProcedure } from "&/server/api/trpc";
import { env } from "&/env";

const GitHubEventSchema = z.object({
  id: z.string(),
  type: z.string(),
  repo: z.object({
    id: z.number(),
    name: z.string(),
    url: z.string(),
  }),
  created_at: z.string(),
  payload: z.any(),
});
const eventsSchema = z.array(GitHubEventSchema);

/**
 * Fetch recent GitHub events for the authenticated user.
 */
export const githubRouter = createTRPCRouter({
  getActivity: protectedProcedure
    .input(z.object({ username: z.string().min(1), perPage: z.number().min(1).max(100).default(20) }))
    .query(async ({ input }) => {
      const response = await fetch(
        `https://api.github.com/users/${input.username}/events/public?per_page=${input.perPage}`,
        {
          headers: {
            Authorization: `Bearer ${env.GITHUB_TOKEN}`,
            Accept: "application/vnd.github.v3+json",
          },
        }
      );
      if (!response.ok) {
        throw new Error(`GitHub API error: ${response.status} ${response.statusText}`);
      }
      const eventsRaw: unknown = await response.json();
      const events = eventsSchema.parse(eventsRaw);
      return events;
    }),
});