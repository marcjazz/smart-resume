import { z } from "zod";
import { createTRPCRouter, protectedProcedure } from "&/server/api/trpc";
import { TRPCError } from "@trpc/server";

const GitHubEventSchema = z.object({
  id: z.string(),
  type: z.string(),
  repo: z.object({
    name: z.string(),
  }),
  payload: z.record(z.any()),
  created_at: z.string(),
});

const GitHubEventsSchema = z.array(GitHubEventSchema);

export const githubRouter = createTRPCRouter({
  sync: protectedProcedure.mutation(async ({ ctx }) => {
    const { session, db } = ctx;
    const userId = session.user.id;

    // 1. Get user's GitHub account and access token from the database
    const account = await db.account.findFirst({
      where: { userId, provider: "github" },
    });

    if (!account?.access_token) {
      throw new TRPCError({
        code: "PRECONDITION_FAILED",
        message: "GitHub account not connected or access token is missing.",
      });
    }

    const user = await db.user.findUnique({
      where: { id: userId },
    });

    if (!user?.name) {
      throw new TRPCError({
        code: "PRECONDITION_FAILED",
        message: "GitHub username not found.",
      });
    }

    // 2. Fetch events from GitHub API
    const response = await fetch(
      `https://api.github.com/users/${user.name}/events?per_page=100`,
      {
        headers: {
          Authorization: `Bearer ${account.access_token}`,
          "User-Agent": "Smart-Resume-Agent",
          Accept: "application/vnd.github.v3+json",
        },
      },
    );

    if (!response.ok) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: `Failed to fetch GitHub events: ${response.statusText}`,
      });
    }

    const eventsJson: unknown = await response.json();
    const events = GitHubEventsSchema.parse(eventsJson);

    // 3. Save events to the database
    const activitiesToCreate = events.map((event) => ({
      type: event.type,
      content: {
        id: event.id,
        repo: event.repo.name,
        payload: event.payload,
        created_at: event.created_at,
      },
      userId: userId,
    }));

    // Use a transaction to ensure atomicity
    await db.$transaction([
      // First, delete all existing activities for the user
      db.activity.deleteMany({
        where: { userId },
      }),
      // Then, create the new ones
      db.activity.createMany({
        data: activitiesToCreate,
      }),
    ]);

    return {
      message: "GitHub activity synced successfully.",
      syncedActivities: events.length,
    };
  }),
});