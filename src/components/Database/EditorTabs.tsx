// Multi-Tab SQL Editor Container

import { useState } from 'react';
import { Button } from '@/components/ui';
import { Plus, X } from 'lucide-react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

export interface EditorTab {
  id: string;
  title: string;
  content: string;
}

interface EditorTabsProps {
  tabs: EditorTab[];
  activeTabId: string;
  onTabChange: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
  onTabAdd: () => void;
  onTabUpdate: (tabId: string, content: string) => void;
  children: (tab: EditorTab) => React.ReactNode;
}

export function EditorTabs({
  tabs,
  activeTabId,
  onTabChange,
  onTabClose,
  onTabAdd,
  onTabUpdate,
  children,
}: EditorTabsProps) {
  const handleClose = (e: React.MouseEvent, tabId: string) => {
    e.stopPropagation();
    onTabClose(tabId);
  };

  return (
    <Tabs value={activeTabId} onValueChange={onTabChange} className="w-full">
      <div className="flex items-center justify-between border-b">
        <TabsList className="h-10">
          {tabs.map((tab) => (
            <TabsTrigger
              key={tab.id}
              value={tab.id}
              className="relative group flex items-center gap-2"
            >
              <span>{tab.title}</span>
              {tabs.length > 1 && (
                <button
                  onClick={(e) => handleClose(e, tab.id)}
                  className="ml-1 hover:bg-muted rounded p-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
                >
                  <X className="w-3 h-3" />
                </button>
              )}
            </TabsTrigger>
          ))}
        </TabsList>

        <Button
          variant="ghost"
          size="sm"
          onClick={onTabAdd}
          className="h-8 mr-2"
        >
          <Plus className="w-4 h-4 mr-1" />
          New Tab
        </Button>
      </div>

      {tabs.map((tab) => (
        <TabsContent key={tab.id} value={tab.id} className="mt-0">
          {children(tab)}
        </TabsContent>
      ))}
    </Tabs>
  );
}
