"use client";

import React from "react";
import { api } from "&/trpc/react";

export default function ResumesPage() {
  const utils = api.useUtils();

  // Queries
  const { data: resumes, isLoading: loadingResumes } =
    api.resume.getAll.useQuery();

  // Mutations
  const syncGithub = api.github.sync.useMutation({
    onSuccess: () => {
      alert("GitHub activities synced successfully!");
      // Optionally, invalidate other queries that depend on activities
    },
    onError: (error) => {
      alert(`Error syncing GitHub: ${error.message}`);
    },
  });

  const generateResume = api.resume.generate.useMutation({
    onSuccess: () => {
      alert("New resume generated successfully!");
      void utils.resume.getAll.invalidate();
    },
    onError: (error) => {
      alert(`Error generating resume: ${error.message}`);
    },
  });

  const exportPdf = api.resume.export.useMutation({
    onSuccess: (updatedResume) => {
      alert(`PDF exported successfully! URL: ${updatedResume.pdfUrl ?? 'N/A'}`);
      void utils.resume.getAll.invalidate();
    },
    onError: (error) => {
      alert(`Error exporting PDF: ${error.message}`);
    },
  });

  return (
    <div className="p-8">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Resumes</h1>
        <div className="flex gap-2">
          <button
            onClick={() => syncGithub.mutate()}
            disabled={syncGithub.isPending}
            className="btn btn-outline"
          >
            {syncGithub.isPending ? "Syncing..." : "Sync GitHub Activities"}
          </button>
          <button
            onClick={() => generateResume.mutate()}
            disabled={generateResume.isPending || syncGithub.isPending}
            className="btn btn-primary"
          >
            {generateResume.isPending ? "Generating..." : "Generate New Resume"}
          </button>
        </div>
      </div>

      <div className="mt-6">
        {loadingResumes ? (
          <p>Loading resumes...</p>
        ) : (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
            {resumes?.map((r) => (
              <div key={r.id} className="card bg-base-100 shadow-xl">
                <div className="card-body">
                  <p className="whitespace-pre-wrap">
                    {(r.content as { summary?: string })?.summary ?? "No summary."}
                  </p>
                  <div className="card-actions justify-end">
                    {r.pdfUrl ? (
                      <a
                        href={r.pdfUrl}
                        target="_blank"
                        className="btn btn-secondary"
                      >
                        View PDF
                      </a>
                    ) : (
                      <button
                        onClick={() => exportPdf.mutate({ id: r.id })}
                        disabled={exportPdf.isPending && exportPdf.variables?.id === r.id}
                        className="btn btn-accent"
                      >
                        {(exportPdf.isPending && exportPdf.variables?.id === r.id) ? "Exporting..." : "Export to PDF"}
                      </button>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
