import { createTRPCRouter, protectedProcedure } from "&/server/api/trpc";
import { getUploadUrl } from "&/server/api/utils/s3";
import { type Prisma } from "@prisma/client";
import { TRPCError } from "@trpc/server";
import { z } from "zod";

// Define a schema for the JSON content of a resume
const ResumeContentSchema = z.object({
  summary: z.string(),
  // could add more fields here later, like experience, education, etc.
});

export const resumeRouter = createTRPCRouter({
  // Procedure to generate a resume summary from activities
  generate: protectedProcedure.mutation(async ({ ctx }) => {
    const { session, db } = ctx;
    const userId = session.user.id;

    const activities =
      await db.activity.findMany({
      where: { userId },
      orderBy: { createdAt: "desc" },
      take: 50,
    });

    if (activities.length === 0) {
      throw new TRPCError({
        code: "NOT_FOUND",
        message: "No activities found to generate a resume from.",
      });
    }

    const rustServiceUrl = "/api/summarize";
    const response = await fetch(rustServiceUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        activities: activities.map((a) => ({
          type: a.type,
          content: typeof a.content === 'string' ? a.content : JSON.stringify(a.content),
        })),
      }),
    });

    if (!response.ok) {
      throw new TRPCError({
        code: "INTERNAL_SERVER_ERROR",
        message: `Failed to get summary from AI service: ${await response.text()}`,
      });
    }

    const summaryResponse = (await response.json()) as { summary: string };

    const newResume = await db.resume.create({
      data: {
        content: {
          summary: summaryResponse.summary,
        },
        userId: userId,
      },
    });

    return newResume;
  }),

  create: protectedProcedure
    .input(z.object({ content: ResumeContentSchema }))
    .mutation(async ({ ctx, input }) => {
      return ctx.db.resume.create({
        data: {
          content: input.content as Prisma.JsonObject,
          user: { connect: { id: ctx.session.user.id } },
        },
      });
    }),

  getUploadUrl: protectedProcedure
    .input(z.object({ key: z.string(), contentType: z.string() }))
    .query(async ({ input }) => {
      return await getUploadUrl(input.key, input.contentType);
    }),

  getAll: protectedProcedure.query(async ({ ctx }) => {
    return await ctx.db.resume.findMany({
      where: { userId: ctx.session.user.id },
      orderBy: { createdAt: "desc" },
    });
  }),

  update: protectedProcedure
    .input(z.object({ id: z.string(), pdfUrl: z.string().url() }))
    .mutation(async ({ ctx, input }) => {
      return await ctx.db.resume.update({
        where: { id: input.id },
        data: { pdfUrl: input.pdfUrl },
      });
    }),
});