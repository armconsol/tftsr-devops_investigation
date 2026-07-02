// SQL autocomplete provider for Monaco Editor

import * as monaco from 'monaco-editor';

export interface TableSchema {
  name: string;
  columns: Array<{
    name: string;
    data_type: string;
  }>;
}

const SQL_KEYWORDS = [
  'SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE', 'CREATE', 'DROP',
  'ALTER', 'TABLE', 'INDEX', 'VIEW', 'JOIN', 'INNER', 'LEFT', 'RIGHT', 'OUTER',
  'ON', 'AND', 'OR', 'NOT', 'IN', 'BETWEEN', 'LIKE', 'IS', 'NULL', 'ORDER',
  'BY', 'GROUP', 'HAVING', 'LIMIT', 'OFFSET', 'UNION', 'DISTINCT', 'AS',
  'COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'CASE', 'WHEN', 'THEN', 'ELSE', 'END',
  'BEGIN', 'COMMIT', 'ROLLBACK', 'TRANSACTION', 'PRIMARY', 'KEY', 'FOREIGN',
  'REFERENCES', 'CONSTRAINT', 'DEFAULT', 'CASCADE', 'SET', 'VALUES',
];

const SQL_FUNCTIONS = [
  'COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'ROUND', 'FLOOR', 'CEIL', 'ABS',
  'UPPER', 'LOWER', 'CONCAT', 'SUBSTRING', 'LENGTH', 'TRIM', 'REPLACE',
  'NOW', 'CURRENT_DATE', 'CURRENT_TIME', 'DATE_FORMAT', 'DATEDIFF',
  'COALESCE', 'NULLIF', 'CAST', 'CONVERT',
];

export class SQLCompletionProvider implements monaco.languages.CompletionItemProvider {
  private schemas: TableSchema[] = [];

  constructor(schemas: TableSchema[] = []) {
    this.schemas = schemas;
  }

  updateSchemas(schemas: TableSchema[]) {
    this.schemas = schemas;
  }

  provideCompletionItems(
    model: monaco.editor.ITextModel,
    position: monaco.Position,
    context: monaco.languages.CompletionContext,
    token: monaco.CancellationToken
  ): monaco.languages.ProviderResult<monaco.languages.CompletionList> {
    const word = model.getWordUntilPosition(position);
    const range = {
      startLineNumber: position.lineNumber,
      endLineNumber: position.lineNumber,
      startColumn: word.startColumn,
      endColumn: word.endColumn,
    };

    const suggestions: monaco.languages.CompletionItem[] = [];

    // Add SQL keywords
    SQL_KEYWORDS.forEach((keyword) => {
      suggestions.push({
        label: keyword,
        kind: monaco.languages.CompletionItemKind.Keyword,
        insertText: keyword,
        range,
        detail: 'SQL Keyword',
      });
    });

    // Add SQL functions
    SQL_FUNCTIONS.forEach((func) => {
      suggestions.push({
        label: func,
        kind: monaco.languages.CompletionItemKind.Function,
        insertText: `${func}()`,
        insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet,
        range,
        detail: 'SQL Function',
      });
    });

    // Add table names
    this.schemas.forEach((table) => {
      suggestions.push({
        label: table.name,
        kind: monaco.languages.CompletionItemKind.Class,
        insertText: table.name,
        range,
        detail: 'Table',
        documentation: `Table with ${table.columns.length} columns`,
      });

      // Add columns for each table
      table.columns.forEach((column) => {
        suggestions.push({
          label: `${table.name}.${column.name}`,
          kind: monaco.languages.CompletionItemKind.Field,
          insertText: `${table.name}.${column.name}`,
          range,
          detail: column.data_type,
          documentation: `${table.name}.${column.name} (${column.data_type})`,
        });
      });
    });

    return {
      suggestions,
      incomplete: false,
    };
  }
}

export function registerSQLLanguage(monaco: typeof import('monaco-editor')) {
  monaco.languages.register({ id: 'sql' });

  monaco.languages.setMonarchTokensProvider('sql', {
    defaultToken: '',
    tokenPostfix: '.sql',
    ignoreCase: true,

    keywords: SQL_KEYWORDS.map((k) => k.toLowerCase()),
    operators: ['=', '>', '<', '!', '~', '?', ':', '+', '-', '*', '/', '&', '|', '^', '%'],
    brackets: [
      { open: '[', close: ']', token: 'delimiter.square' },
      { open: '(', close: ')', token: 'delimiter.parenthesis' },
    ],

    tokenizer: {
      root: [
        { include: '@comments' },
        { include: '@whitespace' },
        { include: '@numbers' },
        { include: '@strings' },
        [/[;,.]/, 'delimiter'],
        [/[()]/, '@brackets'],
        [
          /[\w@#$]+/,
          {
            cases: {
              '@keywords': 'keyword',
              '@default': 'identifier',
            },
          },
        ],
      ],
      comments: [
        [/--+.*/, 'comment'],
        [/\/\*/, { token: 'comment.quote', next: '@comment' }],
      ],
      comment: [
        [/[^*/]+/, 'comment'],
        [/\*\//, { token: 'comment.quote', next: '@pop' }],
        [/./, 'comment'],
      ],
      whitespace: [[/\s+/, 'white']],
      numbers: [[/\d+(\.\d+)?/, 'number']],
      strings: [
        [/'/, { token: 'string', next: '@string' }],
        [/"/, { token: 'string.double', next: '@stringDouble' }],
      ],
      string: [
        [/[^']+/, 'string'],
        [/''/, 'string'],
        [/'/, { token: 'string', next: '@pop' }],
      ],
      stringDouble: [
        [/[^"]+/, 'string.double'],
        [/""/, 'string.double'],
        [/"/, { token: 'string.double', next: '@pop' }],
      ],
    },
  });
}
