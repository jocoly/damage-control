import { invoke } from "@tauri-apps/api/core";

export function startFocusedInputBridge() {
  function handleKeydown() {
    void invoke("record_focused_keypress", {
      eventAtMillis: Date.now(),
    });
  }

  document.addEventListener("keydown", handleKeydown, { capture: true });

  return () => {
    document.removeEventListener("keydown", handleKeydown, { capture: true });
  };
}
