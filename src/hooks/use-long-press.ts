import type { RefCallback } from "react";

type StartEvent = "pointerdown" | "mousedown" | "touchstart";
type EndEvent = "pointerup" | "mouseup" | "touchend" | "contextmenu";

export type LongPressOptions = {
  onStart: () => void;
  onEnd: () => void;
  events?: [StartEvent, EndEvent];
};

export function useLongPress<T extends Element>({
  onStart,
  onEnd,
  events: [startEvent, endEvent] = ["pointerdown", "pointerup"],
}: LongPressOptions) {
  const ref = (element: T | null) => {
    if (!element) return;

    element.addEventListener(startEvent, onStart);
    element.addEventListener(endEvent, onEnd);

    return () => {
      element.removeEventListener(startEvent, onStart);
      element.removeEventListener(endEvent, onEnd);
    };
  };

  return ref as RefCallback<T>;
}
