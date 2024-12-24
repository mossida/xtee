import { isServer, isTauri } from "@/lib/constants";
import { deviceType } from "detect-it";
import { useState } from "react";

function disableDefaults() {
  // Temporary fix until raspberry pi publishes package labwc 0.8.2
  if (deviceType === "hybrid" || deviceType === "touchOnly")
    document.body.style.cursor = "none";

  document.addEventListener(
    "contextmenu",
    (event) => {
      event.preventDefault();

      return false;
    },
    { capture: true },
  );

  document.addEventListener(
    "selectstart",
    (event) => {
      event.preventDefault();

      return false;
    },
    { capture: true },
  );
}

export function useDisableDefaults() {
  if (!isTauri || isServer) return;

  useState(() => disableDefaults());
}
