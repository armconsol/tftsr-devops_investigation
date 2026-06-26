import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { MoreHorizontal, Trash2 } from 'lucide-react';

interface NoteInfo {
  id: string;
  title: string;
  type: 'global' | 'node' | 'cluster';
  resource?: string;
  lastModified: string;
  author: string;
}

interface NoteListProps {
  notes: NoteInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
  onCreate?: () => void;
  onEdit?: (note: NoteInfo) => void;
  onDelete?: (note: NoteInfo) => void;
}

export function NoteList({
  notes,
  onRefresh,
  isLoading,
  onCreate,
  onEdit,
  onDelete,
}: NoteListProps) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Notes</CardTitle>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm" onClick={onRefresh} disabled={isLoading}>
            Refresh
          </Button>
          <Button size="sm" onClick={onCreate}>
            <span className="mr-2 h-4 w-4">+</span>
            New Note
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Title</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Resource</TableHead>
                <TableHead>Last Modified</TableHead>
                <TableHead>Author</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {notes.map((note) => (
                <TableRow key={note.id}>
                  <TableCell className="font-medium">{note.title}</TableCell>
                  <TableCell>
                    <span className={`inline-flex items-center rounded-full px-2 py-1 text-xs font-medium ${
                      note.type === 'global' ? 'bg-blue-100 text-blue-800' :
                      note.type === 'node' ? 'bg-orange-100 text-orange-800' :
                      'bg-purple-100 text-purple-800'
                    }`}>
                      {note.type}
                    </span>
                  </TableCell>
                  <TableCell>{note.resource || '-'}</TableCell>
                  <TableCell>{note.lastModified}</TableCell>
                  <TableCell>{note.author}</TableCell>
                  <TableCell className="text-right">
                    <div className="flex items-center justify-end space-x-2">
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        onClick={() => onEdit?.(note)}
                        title="Edit"
                      >
                        <span className="h-4 w-4 text-xs">✏️</span>
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-red-100 hover:text-red-600"
                        onClick={() => onDelete?.(note)}
                        title="Delete"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                      <button
                        className="rounded-md p-1 hover:bg-accent"
                        title="More"
                      >
                        <MoreHorizontal className="h-4 w-4" />
                      </button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}
