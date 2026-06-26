import React from "react";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui";
import { Badge } from "@/components/ui";
import type { EventInfo } from "@/lib/tauriCommands";

interface EventListProps {
  events: EventInfo[];
  clusterId: string;
  namespace?: string;
}

export function EventList({ events, clusterId: _clusterId, namespace: _namespace }: EventListProps) {
  const getEventTypeColor = (type: string) => {
    switch (type.toLowerCase()) {
      case "normal":
        return "bg-blue-500";
      case "warning":
        return "bg-yellow-500 text-yellow-900";
      default:
        return "bg-gray-500";
    }
  };

  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Type</TableHead>
            <TableHead>Reason</TableHead>
            <TableHead>Object</TableHead>
            <TableHead>Count</TableHead>
            <TableHead>First Seen</TableHead>
            <TableHead>Last Seen</TableHead>
            <TableHead>Message</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {events.length === 0 ? (
            <TableRow>
              <TableCell colSpan={8} className="text-center text-muted-foreground">
                No events found
              </TableCell>
            </TableRow>
          ) : (
            events.map((event) => (
              <TableRow key={event.name}>
                <TableCell className="font-medium">{event.name}</TableCell>
                <TableCell>
                  <Badge className={`${getEventTypeColor(event.event_type)} text-white`}>
                    {event.event_type}
                  </Badge>
                </TableCell>
                <TableCell className="font-medium">{event.reason}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{event.object}</TableCell>
                <TableCell className="text-sm">{event.count}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{event.first_seen}</TableCell>
                <TableCell className="text-sm text-muted-foreground">{event.last_seen}</TableCell>
                <TableCell className="text-sm text-muted-foreground max-w-md truncate" title={event.message}>
                  {event.message}
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </div>
  );
}
