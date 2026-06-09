import { useEffect, useState, RefObject } from "react";

/**
 * Smart positioning hook that determines if a dropdown/menu should flip upward
 * based on available viewport space below the element.
 *
 * @param open - Whether the menu is currently open
 * @param contentRef - Ref to the dropdown content element
 * @returns Whether the menu should flip upward (bottom-full) or stay downward (top-full)
 */
export function useSmartPosition(
  open: boolean,
  contentRef: RefObject<HTMLElement | null>
): boolean {
  const [flipUpward, setFlipUpward] = useState(false);

  useEffect(() => {
    if (!open || !contentRef.current) return;

    const rect = contentRef.current.getBoundingClientRect();
    const viewportHeight = window.innerHeight;
    const spaceBelow = viewportHeight - rect.bottom;

    // If dropdown extends below viewport (less than 20px space), flip upward
    if (spaceBelow < 20) {
      setFlipUpward(true);
    } else {
      setFlipUpward(false);
    }
  }, [open, contentRef]);

  return flipUpward;
}
