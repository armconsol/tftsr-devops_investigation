import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import type { CronJobInfo } from "@/lib/tauriCommands";

interface CronJobListProps {
  cronJobs: CronJobInfo[];
  _clusterId: string;
  _namespace: string;
}

export function CronJobList({ cronJobs, _clusterId, _namespace }: CronJobListProps) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Namespace</TableHead>
            <TableHead>Schedule</TableHead>
            <TableHead>Active</TableHead>
            <TableHead>Last Schedule</TableHead>
            <TableHead>Age</TableHead>
            <TableHead>Labels</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {cronJobs.length === 0 ? (
            <TableRow>
              <TableCell colSpan={7} className="text-center text-muted-foreground">
                No cron jobs found
              </TableCell>
            </TableRow>
          ) : (
            cronJobs.map((cronJob) => (
              <TableRow key={`${cronJob.name}-${cronJob.namespace}`}>
                <TableCell className="font-medium">{cronJob.name}</TableCell>
                <TableCell>{cronJob.namespace}</TableCell>
                <TableCell>{cronJob.schedule}</TableCell>
                <TableCell>{cronJob.active}</TableCell>
                <TableCell>{cronJob.last_schedule}</TableCell>
                <TableCell className="text-muted-foreground">{cronJob.age}</TableCell>
                <TableCell>
                  {Object.entries(cronJob.labels)
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
