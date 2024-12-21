import { deviceType, supportsTouchEvents } from "detect-it";
import type { RefCallback } from "react";

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
  onStart: () => void;
  onEnd: () => void;
};

export function useLongPress<T extends Element>({
  onStart,
  onEnd,
  type: userType,
}: Options) {
  let type = userType;

  if (!type) {
    if (deviceType === "mouseOnly") type = "mouse";
    else if (deviceType === "touchOnly") type = "touch";
    else if (deviceType === "hybrid" && supportsTouchEvents) type = "touch";
    else type = "pointer";
  }

  const ref = (element: T | null) => {
    if (!element) return;

    const endEvents = end[type];

    element.addEventListener(start[type], onStart);
    for (const event of endEvents) element.addEventListener(event, onEnd);

    return () => {
      element.removeEventListener(start[type], onStart);
      for (const event of endEvents) element.removeEventListener(event, onEnd);
    };
  };

  return ref as RefCallback<T>;
}
