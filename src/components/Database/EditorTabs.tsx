// Multi-Tab SQL Editor Container

import { Button } from '@/components/ui';
import { Plus, X } from 'lucide-react';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui';

export interface BaseEditorTab {
  id: string;
  title: string;
  content: string;
}

interface EditorTabsProps<T extends BaseEditorTab> {
  tabs: T[];
  activeTabId: string;
  onTabChange: (tabId: string) => void;
  onTabClose: (tabId: string) => void;
  onTabAdd: () => void;
  onTabUpdate: (tabId: string, content: string) => void;
  children: (tab: T) => React.ReactNode;
}

export function EditorTabs<T extends BaseEditorTab>({
  tabs,
  activeTabId,
  onTabChange,
  onTabClose,
  onTabAdd,
  children,
}: EditorTabsProps<T>) {
  const handleClose = (e: React.MouseEvent, tabId: string) => {
    e.stopPropagation();
    onTabClose(tabId);
  };

  const activeTab = tabs.find((t) => t.id === activeTabId) || tabs[0];

  return (
    <Tabs value={activeTabId} onValueChange={onTabChange}>
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
                <span
                  role="button"
                  onClick={(e) => handleClose(e, tab.id)}
                  className="ml-1 hover:bg-muted rounded p-0.5 opacity-60 hover:opacity-100 transition-opacity inline-flex cursor-pointer"
                >
                  <X className="w-3 h-3" />
                </span>
              )}
            </TabsTrigger>
          ))}
        </TabsList>

        <Button variant="ghost" size="sm" onClick={onTabAdd} className="h-8 mr-2">
          <Plus className="w-4 h-4 mr-1" />
          New Tab
        </Button>
      </div>

      {activeTab && <div key={activeTab.id}>{children(activeTab)}</div>}
    </Tabs>
  );
}
