import { useToast } from "@/components/ui/use-toast";
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
  const { toast } = useToast();

  const debugEvents = [
    "pointerdown",
    "pointerup",
    "pointercancel",
    "mousedown",
    "mouseup",
    "mousecancel",
    "touchstart",
    "touchend",
    "touchcancel",
    "contextmenu",
  ];

  const debugEvent = (event: Event) => {
    toast({
      title: event.type,
      description: event.type,
    });
  };

  const ref = (element: T | null) => {
    if (!element) return;

    element.addEventListener(startEvent, onStart);
    element.addEventListener(endEvent, onEnd);

    for (const event of debugEvents) {
      element.addEventListener(event, debugEvent);
    }

    return () => {
      element.removeEventListener(startEvent, onStart);
      element.removeEventListener(endEvent, onEnd);

      for (const event of debugEvents) {
        element.removeEventListener(event, debugEvent);
      }
    };
  };

  return ref as RefCallback<T>;
}
