import { useState, useEffect } from "react";

export interface ColumnConfig {
  [columnKey: string]: boolean; // true = visible, false = hidden
}

export interface UseColumnConfigReturn {
  columnConfig: ColumnConfig;
  isColumnVisible: (columnKey: string) => boolean;
  toggleColumn: (columnKey: string) => void;
  resetToDefaults: () => void;
  showAllColumns: () => void;
  hideAllColumns: () => void;
}

/**
 * Hook for managing configurable table columns with localStorage persistence
 * @param resourceType - Unique identifier for the resource (e.g., "pods", "deployments")
 * @param defaultConfig - Default column visibility configuration
 */
export function useColumnConfig(
  resourceType: string,
  defaultConfig: ColumnConfig
): UseColumnConfigReturn {
  const storageKey = `column-config-${resourceType}`;

  const [columnConfig, setColumnConfig] = useState<ColumnConfig>(() => {
    try {
      const stored = localStorage.getItem(storageKey);
      if (stored) {
        return { ...defaultConfig, ...JSON.parse(stored) };
      }
    } catch (error) {
      console.error(`Failed to load column config for ${resourceType}:`, error);
    }
    return defaultConfig;
  });

  useEffect(() => {
    try {
      localStorage.setItem(storageKey, JSON.stringify(columnConfig));
    } catch (error) {
      console.error(`Failed to save column config for ${resourceType}:`, error);
    }
  }, [columnConfig, storageKey, resourceType]);

  const isColumnVisible = (columnKey: string): boolean => {
    return columnConfig[columnKey] !== false; // Default to visible if not specified
  };

  const toggleColumn = (columnKey: string) => {
    setColumnConfig((prev) => ({
      ...prev,
      [columnKey]: !prev[columnKey],
    }));
  };

  const resetToDefaults = () => {
    setColumnConfig(defaultConfig);
  };

  const showAllColumns = () => {
    const allVisible = Object.keys(columnConfig).reduce(
      (acc, key) => ({ ...acc, [key]: true }),
      {}
    );
    setColumnConfig(allVisible);
  };

  const hideAllColumns = () => {
    const allHidden = Object.keys(columnConfig).reduce(
      (acc, key) => ({ ...acc, [key]: false }),
      {}
    );
    setColumnConfig(allHidden);
  };

  return {
    columnConfig,
    isColumnVisible,
    toggleColumn,
    resetToDefaults,
    showAllColumns,
    hideAllColumns,
  };
}
