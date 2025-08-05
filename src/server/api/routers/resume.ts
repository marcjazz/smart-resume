import { z } from "zod";
import { getUploadUrl } from "&/server/api/utils/s3";

import {
  createTRPCRouter,
  protectedProcedure,
} from "&/server/api/trpc";
import { generatePdf } from "&/server/api/utils/latex";
import { s3Client } from "&/server/api/utils/s3";
import { PutObjectCommand } from "@aws-sdk/client-s3";
import { env } from "&/env";

export const resumeRouter = createTRPCRouter({
  create: protectedProcedure
    .input(z.object({ content: z.string().min(1) }))
    .mutation(async ({ ctx, input }) => {
      return ctx.db.resume.create({
        data: {
          content: input.content,
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
    .input(z.object({ id: z.number(), url: z.string().url() }))
    .mutation(async ({ ctx, input }) => {
      return await ctx.db.resume.update({
        where: { id: input.id },
        data: { url: input.url },
      });
    }),

  generatePdf: protectedProcedure
    .input(z.object({ id: z.number() }))
    .mutation(async ({ ctx, input }) => {
      const resume = await ctx.db.resume.findUnique({ where: { id: input.id } });
      if (!resume) {
        throw new Error("Resume not found");
      }
      const pdfBuffer = await generatePdf(resume.content);
      const key = `resume_${input.id}.pdf`;
      await s3Client.send(
        new PutObjectCommand({
          Bucket: env.AWS_S3_BUCKET_NAME,
          Key: key,
          Body: pdfBuffer,
          ContentType: "application/pdf",
        })
      );
      const url = `https://${env.AWS_S3_BUCKET_NAME}.s3.${env.AWS_REGION}.amazonaws.com/${key}`;
      await ctx.db.resume.update({ where: { id: input.id }, data: { url } });
      return url;
    }),
});