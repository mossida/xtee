import { deviceType, supportsTouchEvents } from "detect-it";
import { type RefCallback, useCallback, useMemo, useRef } from "react";

const kind = ["mouse", "pointer", "touch"] as const;

const start = {
  mouse: "mousedown",
  pointer: "pointerdown",
  touch: "touchstart",
} as const;

const end = {
  mouse: ["mouseup", "mouseleave"],
  pointer: ["pointerup", "pointerleave"],
  touch: ["touchend", "contextmenu"],
} as const;

type Options = {
  type?: (typeof kind)[number];
  threshold?: number;
  onStart: () => void;
  onEnd: () => void;
};

export function useLongPress<T extends Element>({
  onStart,
  onEnd,
  type: userType,
  threshold = 0,
}: Options) {
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  const type = useMemo(() => {
    if (userType) return userType;

    if (deviceType === "mouseOnly") return "mouse";
    if (deviceType === "touchOnly") return "touch";
    if (deviceType === "hybrid" && supportsTouchEvents) return "touch";
    return "pointer";
  }, [userType]);

  const startCallback = useCallback(() => {
    if (threshold > 0) timerRef.current = setTimeout(onStart, threshold);
    else onStart();
  }, [onStart, threshold]);

  const endCallback = useCallback(() => {
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = null;
    onEnd();
  }, [onEnd]);

  return useCallback(
    (element: T | null) => {
      if (!element) return;

      element.addEventListener(start[type], startCallback);
      for (const event of end[type])
        element.addEventListener(event, endCallback);

      return () => {
        element.removeEventListener(start[type], startCallback);
        for (const event of end[type])
          element.removeEventListener(event, endCallback);
      };
    },
    [startCallback, endCallback, type],
  ) as RefCallback<T>;
}
