// Monaco SQL Editor Component

import { useEffect, useRef } from 'react';
import * as monaco from 'monaco-editor';
import { registerSQLLanguage, SQLCompletionProvider, type TableSchema } from '@/lib/sqlAutocomplete';

interface MonacoSQLEditorProps {
  value: string;
  onChange: (value: string) => void;
  onExecute?: () => void;
  schemas?: TableSchema[];
  height?: string;
  readOnly?: boolean;
}

export function MonacoSQLEditor({
  value,
  onChange,
  onExecute,
  schemas = [],
  height = '400px',
  readOnly = false,
}: MonacoSQLEditorProps) {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const completionProviderRef = useRef<SQLCompletionProvider | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // Register SQL language once
    if (!monaco.languages.getLanguages().some((lang) => lang.id === 'sql')) {
      registerSQLLanguage(monaco);
    }

    // Create completion provider
    completionProviderRef.current = new SQLCompletionProvider(schemas);
    const disposable = monaco.languages.registerCompletionItemProvider(
      'sql',
      completionProviderRef.current
    );

    // Create editor
    editorRef.current = monaco.editor.create(containerRef.current, {
      value,
      language: 'sql',
      theme: 'vs-dark',
      automaticLayout: true,
      minimap: { enabled: true },
      fontSize: 14,
      lineNumbers: 'on',
      renderWhitespace: 'selection',
      wordWrap: 'on',
      folding: true,
      suggest: {
        snippetsPreventQuickSuggestions: false,
      },
      readOnly,
    });

    // Handle value changes
    editorRef.current.onDidChangeModelContent(() => {
      onChange(editorRef.current!.getValue());
    });

    // Bind Ctrl+Enter to execute
    if (onExecute) {
      editorRef.current.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
        onExecute();
      });
    }

    return () => {
      disposable.dispose();
      editorRef.current?.dispose();
    };
  }, []);

  // Update schemas when they change
  useEffect(() => {
    if (completionProviderRef.current) {
      completionProviderRef.current.updateSchemas(schemas);
    }
  }, [schemas]);

  // Update value when prop changes
  useEffect(() => {
    if (editorRef.current && editorRef.current.getValue() !== value) {
      editorRef.current.setValue(value);
    }
  }, [value]);

  return (
    <div
      ref={containerRef}
      style={{ height, width: '100%', border: '1px solid var(--border)' }}
    />
  );
}
