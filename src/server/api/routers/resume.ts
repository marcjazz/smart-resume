import { z } from "zod";
import { getUploadUrl } from "&/server/api/utils/s3";
import { createTRPCRouter, protectedProcedure } from "&/server/api/trpc";
import { TRPCError } from "@trpc/server";
import { Prisma } from "@prisma/client";

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

    const activities = await db.activity.findMany({
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

    // TODO: Move this URL to environment variables
    const rustServiceUrl = "http://127.0.0.1:3000/summarize";
    const response = await fetch(rustServiceUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        activities: activities.map((a) => ({
          r#type: a.type,
          content: a.content,
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

  // Procedure to export a resume to PDF
  export: protectedProcedure
    .input(z.object({ id: z.string() }))
    .mutation(async ({ ctx, input }) => {
      const { session, db } = ctx;
      const userId = session.user.id;

      const resume = await db.resume.findUnique({
        where: { id: input.id, userId },
      });

      if (!resume) {
        throw new TRPCError({ code: "NOT_FOUND", message: "Resume not found." });
      }

      const content = ResumeContentSchema.parse(resume.content);

      // TODO: Move this URL to environment variables
      const rustServiceUrl = "http://127.0.0.1:3000/export-pdf";
      const response = await fetch(rustServiceUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ summary: content.summary }),
      });

      if (!response.ok) {
        throw new TRPCError({
          code: "INTERNAL_SERVER_ERROR",
          message: `Failed to export PDF from service: ${await response.text()}`,
        });
      }

      const exportResponse = (await response.json()) as { pdf_url: string };

      const updatedResume = await db.resume.update({
        where: { id: input.id },
        data: { pdfUrl: exportResponse.pdf_url },
      });

      return updatedResume;
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