export const ACTIVE_VISUAL_UPDATE_MS = 100;
export const IDLE_AFTER_MS = 30_000;
export const IDLE_VISUAL_CHECK_MS = 1_000;

export type InputSnapshot = {
  influence: number;
  xp: number;
  level: number;
  xp_for_current_level: number;
  xp_for_next_level: number;
  keys: number;
  clicks: number;
  bonus_influence: number;
  power_event_sequence: number;
  last_power_event_at_millis: number;
  last_power_event_amount: number;
  inventory_item_ids: string[];
  last_input_at_millis: number;
};

export type VisualFrame = InputSnapshot & {
  isIdle: boolean;
  lastInputAgeMs: number | null;
};

type VisualUpdateLoopOptions = {
  readSnapshot: () => Promise<InputSnapshot>;
  onFrame: (frame: VisualFrame) => void;
  onError: () => void;
};

export function startVisualUpdateLoop({
  readSnapshot,
  onFrame,
  onError,
}: VisualUpdateLoopOptions) {
  let stopped = false;
  let timeoutId: number | undefined;
  let wasIdle: boolean | undefined;

  async function tick() {
    try {
      const snapshot = await readSnapshot();
      const lastInputAgeMs =
        snapshot.last_input_at_millis === 0
          ? null
          : Math.max(0, Date.now() - snapshot.last_input_at_millis);
      const isIdle = lastInputAgeMs === null || lastInputAgeMs >= IDLE_AFTER_MS;

      if (!isIdle || wasIdle !== isIdle) {
        onFrame({
          ...snapshot,
          isIdle,
          lastInputAgeMs,
        });
      }

      wasIdle = isIdle;
      scheduleNext(isIdle ? IDLE_VISUAL_CHECK_MS : ACTIVE_VISUAL_UPDATE_MS);
    } catch {
      onError();
      scheduleNext(IDLE_VISUAL_CHECK_MS);
    }
  }

  function scheduleNext(delay: number) {
    if (stopped) {
      return;
    }

    timeoutId = window.setTimeout(tick, delay);
  }

  tick();

  return () => {
    stopped = true;

    if (timeoutId !== undefined) {
      window.clearTimeout(timeoutId);
    }
  };
}
