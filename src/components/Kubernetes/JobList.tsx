import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { JobInfo } from "@/lib/tauriCommands";

interface JobListProps {
  jobs: JobInfo[];
  _clusterId: string;
  _namespace: string;
}

export function JobList({ jobs, _clusterId, _namespace }: JobListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Completions</TableHead>
            <TableHead>Duration</TableHead>
            <TableHead>Age</TableHead>
            <TableHead>Labels</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {jobs.length === 0 ? (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                No jobs found
              </TableCell>
            </TableRow>
          ) : (
            jobs.map((job) => (
              <TableRow key={`${job.name}-${job.namespace}`}>
                <TableCell className="font-medium">{job.name}</TableCell>
                <TableCell>{job.namespace}</TableCell>
                <TableCell>{job.completions}</TableCell>
                <TableCell>{job.duration}</TableCell>
                <TableCell className="text-muted-foreground">{job.age}</TableCell>
                <TableCell>
                  {Object.entries(job.labels)
                    .map(([k, v]) => `${k}=${v}`)
                    .join(", ")}
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
