import { NavLink, useLocation } from "react-router-dom";
import { ChevronDown, ChevronRight } from "lucide-react";
import type { LucideIcon } from "lucide-react";

export interface NavItem {
  /** Stable identifier used to track expand/collapse state — not a route. */
  id: string;
  /** Route path. Omit for a group that is purely a collapsible container. */
  to?: string;
  label: string;
  icon?: LucideIcon;
  children?: NavItem[];
}

interface SidebarNavProps {
  items: NavItem[];
  collapsed: boolean;
  expandedSections: string[];
  onToggleSection: (id: string) => void;
  depth?: number;
}

function isGroupActive(item: NavItem, pathname: string): boolean {
  if (item.to && pathname.startsWith(item.to)) return true;
  return !!item.children?.some((child) => isGroupActive(child, pathname));
}

const itemClasses = (isActive: boolean) =>
  `flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
    isActive
      ? "bg-primary text-primary-foreground"
      : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
  }`;

/**
 * Recursive sidebar navigation renderer. Groups (items with `children`) are
 * collapsible buttons; leaves are `NavLink`s. Expand/collapse state is
 * lifted to the caller via `expandedSections`/`onToggleSection` so it can be
 * shared across arbitrarily nested groups (e.g. Tools > Proxmox > VMs).
 */
export function SidebarNav({
  items,
  collapsed,
  expandedSections,
  onToggleSection,
  depth = 0,
}: SidebarNavProps) {
  const location = useLocation();

  return (
    <div className={depth > 0 ? "ml-4 space-y-1 pl-4 border-l border-muted" : "space-y-1"}>
      {items.map((item) => {
        if (item.children && item.children.length > 0) {
          const isExpanded = expandedSections.includes(item.id);
          const isActive = isGroupActive(item, location.pathname);
          return (
            <div key={item.id}>
              <button
                type="button"
                onClick={() => onToggleSection(item.id)}
                aria-expanded={isExpanded}
                className={`w-full ${itemClasses(isActive)}`}
              >
                {item.icon && <item.icon className="w-4 h-4 shrink-0" />}
                {!collapsed && <span>{item.label}</span>}
                {!collapsed &&
                  (isExpanded ? (
                    <ChevronDown className="w-3 h-3 ml-auto" />
                  ) : (
                    <ChevronRight className="w-3 h-3 ml-auto" />
                  ))}
              </button>
              {!collapsed && isExpanded && (
                <SidebarNav
                  items={item.children}
                  collapsed={collapsed}
                  expandedSections={expandedSections}
                  onToggleSection={onToggleSection}
                  depth={depth + 1}
                />
              )}
            </div>
          );
        }

        return (
          <NavLink
            key={item.id}
            to={item.to ?? "#"}
            end={item.to === "/"}
            className={({ isActive }) => itemClasses(isActive)}
          >
            {item.icon ? (
              <item.icon className="w-4 h-4 shrink-0" />
            ) : (
              depth > 0 && <span className="w-4 h-4 shrink-0" />
            )}
            {!collapsed && <span>{item.label}</span>}
          </NavLink>
        );
      })}
    </div>
  );
}
