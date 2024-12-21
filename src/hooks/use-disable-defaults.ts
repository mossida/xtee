import { isServer, isTauri } from "@/lib/constants";
import { useState } from "react";

function disableDefaults() {
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
