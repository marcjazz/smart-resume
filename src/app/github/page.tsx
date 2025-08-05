"use client";
import React, { useState } from "react";
import { api } from "&/trpc/react";

export default function GitHubActivityPage() {
  const [username, setUsername] = useState("");
  const {
    data: events,
    refetch,
    isFetching,
  } = api.github.getActivity.useQuery(
    { username, perPage: 20 },
    { enabled: false }
  );

  const handleFetch = (e: React.FormEvent) => {
    e.preventDefault();
    if (!username.trim()) return;
    void refetch();
  };

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-4">GitHub Activity</h1>
      <form onSubmit={handleFetch} className="mb-4 flex gap-2">
        <input
          type="text"
          placeholder="GitHub Username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          className="border rounded px-2 py-1 flex-1"
        />
        <button
          type="submit"
          className="bg-blue-600 text-white px-4 py-1 rounded"
          disabled={isFetching}
        >
          {isFetching ? "Loading..." : "Fetch"}
        </button>
      </form>
      {events && (
        <ul className="space-y-2">
          {events.map((ev) => (
            <li
              key={ev.id}
              className="border rounded p-2 flex justify-between"
            >
              <span>{ev.type}</span>
              <span className="font-mono">{ev.repo.name}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}