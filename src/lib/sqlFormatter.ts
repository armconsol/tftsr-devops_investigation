// SQL formatting utilities using sql-formatter

import { format } from 'sql-formatter';

export interface FormatOptions {
  language?: 'sql' | 'mysql' | 'postgresql' | 'plsql' | 'db2' | 'n1ql';
  indent?: string;
  uppercase?: boolean;
  linesBetweenQueries?: number;
}

export function formatSQL(sql: string, options: FormatOptions = {}): string {
  try {
    return format(sql, {
      language: options.language || 'sql',
      indent: options.indent || '  ',
      uppercase: options.uppercase !== false,
      linesBetweenQueries: options.linesBetweenQueries || 2,
    });
  } catch (error) {
    console.error('SQL formatting error:', error);
    return sql; // Return original if formatting fails
  }
}

export function formatSQLForDatabase(sql: string, dbType: string): string {
  const languageMap: Record<string, string> = {
    postgres: 'postgresql',
    mysql: 'mysql',
    mongodb: 'sql', // MongoDB uses JSON, but we format SQL-like queries
    cassandra: 'plsql',
    redis: 'sql',
  };

  return formatSQL(sql, {
    language: (languageMap[dbType] || 'sql') as any,
    uppercase: true,
  });
}

export function minifySQL(sql: string): string {
  return sql
    .replace(/\s+/g, ' ')
    .replace(/\s*,\s*/g, ',')
    .replace(/\s*\(\s*/g, '(')
    .replace(/\s*\)\s*/g, ')')
    .trim();
}

export function extractTableNames(sql: string): string[] {
  const tables: string[] = [];
  const fromMatch = sql.match(/FROM\s+([a-zA-Z0-9_."]+)/gi);
  const joinMatch = sql.match(/JOIN\s+([a-zA-Z0-9_."]+)/gi);

  if (fromMatch) {
    fromMatch.forEach((match) => {
      const table = match.replace(/FROM\s+/i, '').trim();
      tables.push(table);
    });
  }

  if (joinMatch) {
    joinMatch.forEach((match) => {
      const table = match.replace(/JOIN\s+/i, '').trim();
      tables.push(table);
    });
  }

  return [...new Set(tables)];
}
