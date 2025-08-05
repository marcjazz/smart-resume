"use client";

import React, { useState } from "react";
import { api } from "&/trpc/react";

export default function ResumesPage() {
  const [content, setContent] = useState("");
  const [file, setFile] = useState<File | null>(null);
  const utils = api.useUtils();

  const { data: resumes, isLoading: loadingResumes } =
    api.resume.getAll.useQuery();

  const createResume = api.resume.create.useMutation({
    onSuccess: () => utils.resume.getAll.invalidate(),
  });

  const updateUrl = api.resume.update.useMutation({
    onSuccess: () => utils.resume.getAll.invalidate(),
  });

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    if (!content.trim()) return;
    createResume.mutate({ content });
    setContent("");
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files?.[0];
    if (selected) {
      setFile(selected);
    }
  };

  const handleUpload = async (resumeId: number) => {
    if (!file) return;
    const key = `resume_${resumeId}_${file.name}`;
    const contentType = file.type;
    const signedUrl = await utils.resume.getUploadUrl.fetch({
      key,
      contentType,
    });
    await fetch(signedUrl, { method: "PUT", body: file });
    const publicUrl = `https://${process.env.NEXT_PUBLIC_AWS_S3_BUCKET_NAME}.s3.${process.env.NEXT_PUBLIC_AWS_REGION}.amazonaws.com/${key}`;
    updateUrl.mutate({ id: resumeId, url: publicUrl });
    setFile(null);
  };

  return (
    <div className="p-6">
      <h1 className="mb-4 text-2xl font-bold">Resumes</h1>

      <form onSubmit={handleCreate} className="mb-6 flex flex-col gap-2">
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder="Enter resume content"
          className="w-full rounded border p-2"
          rows={5}
        />
        <button
          type="submit"
          disabled={createResume.isPending}
          className="self-start rounded bg-blue-600 px-4 py-2 text-white"
        >
          {createResume.isPending ? "Creating..." : "Create Resume"}
        </button>
      </form>

      {loadingResumes ? (
        <p>Loading resumes...</p>
      ) : (
        <ul className="space-y-4">
          {resumes?.map((r) => (
            <li key={r.id} className="flex flex-col gap-2 rounded border p-4">
              <p className="whitespace-pre-wrap">{r.content}</p>
              {r.url ? (
                <a
                  href={r.url}
                  target="_blank"
                  className="text-blue-600 underline"
                >
                  View PDF
                </a>
              ) : (
                <div className="flex items-center gap-2">
                  <input
                    type="file"
                    accept="application/pdf"
                    onChange={handleFileChange}
                  />
                  <button
                    onClick={() => handleUpload(r.id)}
                    disabled={!file}
                    className="rounded bg-green-600 px-3 py-1 text-white"
                  >
                    Upload PDF
                  </button>
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
