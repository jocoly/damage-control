<script lang="ts">
  import { dev } from "$app/environment";
  import { LogicalSize } from "@tauri-apps/api/dpi";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, tick } from "svelte";
  import { startFocusedInputBridge } from "$lib/focusedInputBridge";
  import {
    isPromotionLevel,
    titleForLevel,
    titleForLevelAndInventory,
    titleHoverText,
  } from "$lib/progressionCatalog";
  import { shopItems, type ShopItem } from "$lib/shopCatalog";
  import {
    startVisualUpdateLoop,
    type InputSnapshot,
    type VisualFrame,
  } from "$lib/visualUpdateLoop";

  let counts = $state<VisualFrame>({
    influence: 0,
    xp: 0,
    level: 1,
    xp_for_current_level: 0,
    xp_for_next_level: 500,
    keys: 0,
    clicks: 0,
    bonus_influence: 0,
    power_event_sequence: 0,
    last_power_event_at_millis: 0,
    last_power_event_amount: 0,
    inventory_item_ids: [],
    last_input_at_millis: 0,
    isIdle: true,
    lastInputAgeMs: null,
  });
  let isDetailsOpen = $state(false);
  let isShopOpen = $state(false);
  let isInventoryOpen = $state(false);
  let isSettingsOpen = $state(false);
  let isDevToolsOpen = $state(false);
  let presentationMode = $state("");
  let devMessage = $state("");
  let purchaseMessage = $state("");
  let resetMessage = $state("");
  let settingsMessage = $state("");
  let appSettings = $state<AppSettings>({
    run_on_startup: false,
    always_on_top: false,
    show_taskbar_icon: true,
  });
  let pendingPurchaseId = $state<string | null>(null);
  let confirmingPurchaseId = $state<string | null>(null);
  let isConfirmingReset = $state(false);
  let isResetting = $state(false);
  let isPowerProcVisible = $state(false);
  let powerProcAmount = $state(0);
  let hasLoadedInitialSnapshot = false;
  let lastSeenLevel = 1;
  let lastSeenPowerEventSequence = 0;
  let notifications = $state<GameNotification[]>([]);
  let activeNotification = $state<GameNotification | null>(null);
  let panelElement: HTMLElement;
  let lastWindowWidth = 0;
  let lastWindowHeight = 0;
  let resizeFrameId: number | undefined;
  let powerProcTimeoutId: number | undefined;

  type PurchaseResult = {
    status:
      | "purchased"
      | "already_owned"
      | "not_enough_influence"
      | "locked"
      | "unknown_item";
    snapshot: InputSnapshot;
  };

  type GameNotification = {
    id: string;
    title: string;
    body: string;
    level: number;
  };

  type RuntimeConfig = {
    presentation_mode: string;
  };

  type AppSettings = {
    run_on_startup: boolean;
    always_on_top: boolean;
    show_taskbar_icon: boolean;
  };

  const numberFormatter = new Intl.NumberFormat("en-US");
  const royalContractId = "royal_contract";

  function isOwned(itemId: string) {
    return counts.inventory_item_ids.includes(itemId);
  }

  function ownsRoyalContract() {
    return isOwned(royalContractId);
  }

  function applySnapshot(snapshot: InputSnapshot, options = { syncLevelTracker: false }) {
    counts = {
      ...counts,
      ...snapshot,
    };

    if (options.syncLevelTracker) {
      hasLoadedInitialSnapshot = true;
      lastSeenLevel = snapshot.level;
      lastSeenPowerEventSequence = snapshot.power_event_sequence;
      isPowerProcVisible = false;
    } else {
      updatePowerProcFeedback(snapshot);
    }
  }

  function applyVisualFrame(frame: VisualFrame) {
    const previousLevel = lastSeenLevel;

    counts = frame;

    if (!hasLoadedInitialSnapshot) {
      hasLoadedInitialSnapshot = true;
      lastSeenLevel = frame.level;
      lastSeenPowerEventSequence = frame.power_event_sequence;
      return;
    }

    if (frame.level > previousLevel) {
      enqueueLevelNotifications(previousLevel + 1, frame.level);
    }

    updatePowerProcFeedback(frame);
    lastSeenLevel = frame.level;
  }

  function updatePowerProcFeedback(snapshot: InputSnapshot) {
    if (
      hasLoadedInitialSnapshot &&
      snapshot.power_event_sequence > lastSeenPowerEventSequence
    ) {
      showPowerProcFeedback(snapshot.last_power_event_amount);
    }

    lastSeenPowerEventSequence = snapshot.power_event_sequence;
  }

  function showPowerProcFeedback(amount: number) {
    powerProcAmount = amount;
    isPowerProcVisible = true;

    if (powerProcTimeoutId !== undefined) {
      window.clearTimeout(powerProcTimeoutId);
    }

    powerProcTimeoutId = window.setTimeout(() => {
      isPowerProcVisible = false;
    }, 1_600);
  }

  function enqueueLevelNotifications(startLevel: number, endLevel: number) {
    const nextNotifications: GameNotification[] = [];

    for (let level = startLevel; level <= endLevel; level += 1) {
      nextNotifications.push({
        id: `level-${level}-${Date.now()}`,
        title: "Level up!",
        body: `You reached Level ${level}.`,
        level,
      });

      if (isPromotionLevel(level)) {
        nextNotifications.push({
          id: `promotion-${level}-${Date.now()}`,
          title: "Promotion!",
          body: titleForLevel(level).name,
          level,
        });
      }
    }

    notifications = [...notifications, ...nextNotifications];
  }

  async function devAddInfluence(amount: number) {
    devMessage = "";

    try {
      const snapshot = await invoke<InputSnapshot>("dev_add_influence", { amount });
      applySnapshot(snapshot);
      devMessage = `Added ${formatNumber(amount)} Influence.`;
    } catch {
      devMessage = "Dev action failed.";
    }
  }

  function openNextNotification() {
    if (activeNotification !== null || notifications.length === 0) {
      return;
    }

    const [nextNotification, ...remainingNotifications] = notifications;
    activeNotification = nextNotification;
    notifications = remainingNotifications;
  }

  function dismissNotification() {
    if (notifications.length > 0) {
      const [nextNotification, ...remainingNotifications] = notifications;
      activeNotification = nextNotification;
      notifications = remainingNotifications;
      return;
    }

    activeNotification = null;
  }

  function levelProgressPercent() {
    const levelSpan = counts.xp_for_next_level - counts.xp_for_current_level;

    if (levelSpan <= 0) {
      return 0;
    }

    return Math.min(
      100,
      Math.max(0, ((counts.xp - counts.xp_for_current_level) / levelSpan) * 100),
    );
  }

  function levelProgressText() {
    return `${formatNumber(counts.xp)} / ${formatNumber(counts.xp_for_next_level)} XP`;
  }

  function formatNumber(value: number) {
    return numberFormatter.format(value);
  }

  function currentTitle() {
    return titleForLevelAndInventory(counts.level, counts.inventory_item_ids);
  }

  function isUnlocked(item: ShopItem) {
    return counts.level >= item.requiredLevel;
  }

  function closeShopState() {
    purchaseMessage = "";
    confirmingPurchaseId = null;
  }

  function closeAllMenus() {
    if (isShopOpen) {
      closeShopState();
    }

    isDetailsOpen = false;
    isShopOpen = false;
    isInventoryOpen = false;
    isSettingsOpen = false;
    isDevToolsOpen = false;
  }

  function toggleDetails() {
    const shouldOpen = !isDetailsOpen;
    closeAllMenus();
    isDetailsOpen = shouldOpen;
  }

  function toggleShop() {
    const shouldOpen = !isShopOpen;
    closeAllMenus();
    isShopOpen = shouldOpen;

    if (!isShopOpen) {
      closeShopState();
    }
  }

  function toggleInventory() {
    const shouldOpen = !isInventoryOpen;
    closeAllMenus();
    isInventoryOpen = shouldOpen;
  }

  function toggleSettings() {
    const shouldOpen = !isSettingsOpen;
    closeAllMenus();
    isSettingsOpen = shouldOpen;
  }

  function toggleDevTools() {
    const shouldOpen = !isDevToolsOpen;
    closeAllMenus();
    isDevToolsOpen = shouldOpen;
  }

  async function updateSetting(key: keyof AppSettings, value: boolean) {
    const previousSettings = appSettings;
    const nextSettings = {
      ...appSettings,
      [key]: value,
    };

    settingsMessage = "";
    appSettings = nextSettings;

    try {
      appSettings = await invoke<AppSettings>("update_app_settings", {
        settings: nextSettings,
      });
    } catch {
      appSettings = previousSettings;
      settingsMessage = "Setting update failed.";
    }
  }

  function availableShopItems() {
    return shopItems.filter(
      (item) =>
        isUnlocked(item) &&
        !isOwned(item.id) &&
        (item.id === royalContractId || ownsRoyalContract()),
    );
  }

  function ownedFunctionalItems() {
    return shopItems.filter((item) => item.category !== "org_chart" && isOwned(item.id));
  }

  function requestPurchaseConfirmation(item: ShopItem) {
    if (!isUnlocked(item) || isOwned(item.id) || pendingPurchaseId !== null) {
      return;
    }

    purchaseMessage = "";
    resetMessage = "";
    confirmingPurchaseId = item.id;
  }

  async function confirmPurchase(item: ShopItem) {
    if (!isUnlocked(item) || isOwned(item.id) || pendingPurchaseId !== null) {
      return;
    }

    pendingPurchaseId = item.id;
    confirmingPurchaseId = null;
    purchaseMessage = "";

    try {
      const result = await invoke<PurchaseResult>("purchase_shop_item", {
        itemId: item.id,
      });

      applySnapshot(result.snapshot);

      purchaseMessage =
        result.status === "purchased"
          ? `${item.name} purchased.`
          : result.status === "not_enough_influence"
            ? "Not enough Influence."
            : result.status === "already_owned"
              ? "Already purchased."
              : result.status === "locked"
                ? "Level too low."
                : "Item unavailable.";
    } catch {
      purchaseMessage = "Purchase failed.";
    } finally {
      pendingPurchaseId = null;
    }
  }

  function cancelPurchase() {
    confirmingPurchaseId = null;
  }

  function requestResetConfirmation() {
    purchaseMessage = "";
    resetMessage = "";
    isConfirmingReset = true;
  }

  async function confirmReset() {
    if (isResetting) {
      return;
    }

    isResetting = true;
    isConfirmingReset = false;
    purchaseMessage = "";
    resetMessage = "";

    try {
      const snapshot = await invoke<InputSnapshot>("reset_progress");
      applySnapshot(snapshot, { syncLevelTracker: true });
      confirmingPurchaseId = null;
      notifications = [];
      activeNotification = null;
      resetMessage = "Progress reset.";
    } catch {
      resetMessage = "Reset failed.";
    } finally {
      isResetting = false;
    }
  }

  function cancelReset() {
    isConfirmingReset = false;
  }

  async function exitApp() {
    await invoke("exit_app");
  }

  async function startWindowDrag(event: PointerEvent) {
    if (event.button !== 0 || isInteractiveTarget(event.target)) {
      return;
    }

    await getCurrentWindow().startDragging();
  }

  function suppressContextMenu(event: MouseEvent) {
    event.preventDefault();
  }

  function isInteractiveTarget(target: EventTarget | null) {
    return (
      target instanceof Element &&
      target.closest("button, a, input, select, textarea, [role='button']") !== null
    );
  }

  function scheduleWindowResize() {
    if (resizeFrameId !== undefined) {
      window.cancelAnimationFrame(resizeFrameId);
    }

    resizeFrameId = window.requestAnimationFrame(() => {
      void resizeWindowToPanel();
    });
  }

  async function resizeWindowToPanel() {
    await tick();

    if (!panelElement) {
      return;
    }

    const width = Math.ceil(panelElement.offsetWidth);
    const height = Math.min(560, Math.max(120, Math.ceil(panelElement.scrollHeight)));

    if (Math.abs(width - lastWindowWidth) < 2 && Math.abs(height - lastWindowHeight) < 2) {
      return;
    }

    lastWindowWidth = width;
    lastWindowHeight = height;

    await getCurrentWindow().setSize(new LogicalSize(width, height));
  }

  onMount(() => {
    void invoke<RuntimeConfig>("get_runtime_config").then((config) => {
      presentationMode = config.presentation_mode;
    });
    void invoke<AppSettings>("get_app_settings").then((settings) => {
      appSettings = settings;
    });

    const stopFocusedInputBridge = startFocusedInputBridge();
    const stopVisualUpdateLoop = startVisualUpdateLoop({
      readSnapshot: () => invoke<InputSnapshot>("get_input_counts"),
      onFrame: (frame) => {
        applyVisualFrame(frame);
      },
      onError: () => {},
    });
    const resizeObserver = new ResizeObserver(scheduleWindowResize);

    if (panelElement) {
      resizeObserver.observe(panelElement);
      scheduleWindowResize();
    }

    return () => {
      stopFocusedInputBridge();
      stopVisualUpdateLoop();
      resizeObserver.disconnect();

      if (resizeFrameId !== undefined) {
        window.cancelAnimationFrame(resizeFrameId);
      }

      if (powerProcTimeoutId !== undefined) {
        window.clearTimeout(powerProcTimeoutId);
      }
    };
  });
</script>

<main>
  <section
    bind:this={panelElement}
    class="panel"
    aria-label="Input progress"
    onpointerdown={startWindowDrag}
    oncontextmenu={suppressContextMenu}
  >
    <div class="mode-labels" aria-label="Active modes">
      {#if dev}
        <span class="dev-mode-label">DEV MODE</span>
      {/if}
      {#if presentationMode === "gui"}
        <span>GUI MODE</span>
      {/if}
    </div>

    <div class="titlebar">
      <p>Knight Shift</p>
      <div class="title-actions">
        {#if notifications.length > 0}
          <button
            class="notification-button"
            type="button"
            aria-label={`${notifications.length} notification${notifications.length === 1 ? "" : "s"} ready`}
            onclick={openNextNotification}
          >
            !
          </button>
        {/if}
        <button class="exit-button" type="button" aria-label="Exit" onclick={exitApp}>Exit</button>
      </div>
    </div>

    <div class="counter">
      <p class="label">Influence</p>
      <div class="influence-row">
        <p class="influence">{formatNumber(counts.influence)}</p>
        {#if isPowerProcVisible}
          <span
            class="power-proc-dot"
            aria-label={`Power upgrade triggered for ${formatNumber(powerProcAmount)} Influence`}
          >
            +{formatNumber(powerProcAmount)}
          </span>
        {/if}
      </div>
      <div class="title-rank" aria-label={`Title: ${currentTitle().name}`}>
        <p>{currentTitle().name}</p>
        <span class="title-tooltip" role="tooltip">{titleHoverText(currentTitle())}</span>
      </div>
      <div class="level-progress" aria-label="Level progress">
        <div class="progress-summary">
          <span>Level {counts.level}</span>
        </div>
        <div class="level-meter">
          <span style={`width: ${levelProgressPercent()}%`}></span>
        </div>
        <span class="xp-tooltip" role="tooltip">{levelProgressText()}</span>
      </div>
    </div>

    {#if activeNotification}
      <section class="notification-popup" aria-label="Game notification">
        <p class="notification-title">{activeNotification.title}</p>
        <p>{activeNotification.body}</p>
        <button type="button" onclick={dismissNotification}>
          {notifications.length > 0 ? "Next" : "Close"}
        </button>
      </section>
    {/if}

    <div class="actions">
      <button
        type="button"
        aria-expanded={isDetailsOpen}
        aria-controls="details"
        class:active-menu={isDetailsOpen}
        onclick={toggleDetails}
      >
        Details
      </button>

      <button
        type="button"
        aria-expanded={isShopOpen}
        aria-controls="shop-list"
        class:active-menu={isShopOpen}
        onclick={toggleShop}
      >
        Shop
      </button>

      <button
        type="button"
        aria-expanded={isInventoryOpen}
        aria-controls="inventory"
        class:active-menu={isInventoryOpen}
        onclick={toggleInventory}
      >
        Inventory
      </button>

      <button
        type="button"
        aria-expanded={isSettingsOpen}
        aria-controls="settings"
        class:active-menu={isSettingsOpen}
        onclick={toggleSettings}
      >
        Settings
      </button>

      {#if dev}
        <button
          type="button"
          aria-expanded={isDevToolsOpen}
          aria-controls="dev-tools"
          class:active-menu={isDevToolsOpen}
          onclick={toggleDevTools}
        >
          Dev
        </button>
      {/if}
    </div>

    {#if dev && isDevToolsOpen}
      <section id="dev-tools" class="dev-tools" aria-label="Developer tools">
        <button type="button" onclick={() => devAddInfluence(100)}>+100 Influence</button>
        <button type="button" onclick={() => devAddInfluence(1000)}>+1,000 Influence</button>
        <button type="button" onclick={() => devAddInfluence(10000)}>+10,000 Influence</button>
        <button type="button" onclick={() => devAddInfluence(100000)}>
          +100,000 Influence
        </button>
        <button type="button" onclick={() => devAddInfluence(1000000)}>
          +1,000,000 Influence
        </button>
        {#if devMessage}
          <p>{devMessage}</p>
        {/if}
      </section>
    {/if}

    {#if isDetailsOpen}
      <div id="details" class="details">
        <dl>
          <div>
            <dt>Keys</dt>
            <dd>{formatNumber(counts.keys)}</dd>
          </div>
          <div>
            <dt>Clicks</dt>
            <dd>{formatNumber(counts.clicks)}</dd>
          </div>
        </dl>
      </div>
    {/if}

    {#if isShopOpen}
      <section id="shop-list" class="shop" aria-label="Shop">
        {#if availableShopItems().length > 0}
          <ul>
            {#each availableShopItems() as item}
              <li>
                <button
                  class="shop-item"
                  type="button"
                  disabled={pendingPurchaseId !== null}
                  onclick={() => requestPurchaseConfirmation(item)}
                >
                  <span>{item.name}</span>
                  <span>
                    {#if pendingPurchaseId === item.id}
                      Purchasing
                    {:else}
                      {formatNumber(item.cost)} Influence
                    {/if}
                  </span>
                  <small>{item.effect}</small>
                </button>
                {#if confirmingPurchaseId === item.id}
                  <div class="confirm-purchase" aria-label="Confirm purchase">
                    <p>Spend {formatNumber(item.cost)} Influence?</p>
                    <div>
                      <button type="button" onclick={() => confirmPurchase(item)}>Confirm</button>
                      <button type="button" onclick={cancelPurchase}>Cancel</button>
                    </div>
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        {:else}
          <p class="empty-panel-message">No available shop items.</p>
        {/if}
      </section>
    {/if}

    {#if isInventoryOpen}
      <section id="inventory" class="inventory" aria-label="Inventory">
        <div class="inventory-category">
          <p class="inventory-heading">Items</p>
          {#if ownedFunctionalItems().length > 0}
            <ul>
              {#each ownedFunctionalItems() as item}
                <li>
                  <p>{item.name}</p>
                  <small>{item.effect}</small>
                </li>
              {/each}
            </ul>
          {:else}
            <p class="empty-panel-message">No items owned.</p>
          {/if}
        </div>

        <div class="inventory-category">
          <p class="inventory-heading">Cosmetics</p>
          <p class="empty-panel-message">No cosmetics owned.</p>
        </div>
      </section>
    {/if}

    {#if isSettingsOpen}
      <section id="settings" class="settings" aria-label="Settings">
        <label class="settings-toggle">
          <span>
            <strong>Run on startup</strong>
            <small>Launch Knight Shift when you sign in.</small>
          </span>
          <input
            type="checkbox"
            checked={appSettings.run_on_startup}
            onchange={(event) =>
              updateSetting("run_on_startup", event.currentTarget.checked)}
          />
        </label>

        <label class="settings-toggle">
          <span>
            <strong>Always on top</strong>
            <small>Keep the window above other windows.</small>
          </span>
          <input
            type="checkbox"
            checked={appSettings.always_on_top}
            onchange={(event) =>
              updateSetting("always_on_top", event.currentTarget.checked)}
          />
        </label>

        <label class="settings-toggle">
          <span>
            <strong>Show taskbar icon</strong>
            <small>Display Knight Shift in the Windows taskbar.</small>
          </span>
          <input
            type="checkbox"
            checked={appSettings.show_taskbar_icon}
            onchange={(event) =>
              updateSetting("show_taskbar_icon", event.currentTarget.checked)}
          />
        </label>

        {#if settingsMessage}
          <p class="settings-message">{settingsMessage}</p>
        {/if}
      </section>
    {/if}

    {#if purchaseMessage}
      <p class="purchase-message">{purchaseMessage}</p>
    {/if}

    <button
      class="reset-toggle"
      type="button"
      disabled={isResetting}
      onclick={requestResetConfirmation}
    >
      {isResetting ? "Resetting" : "Reset"}
    </button>

    {#if isConfirmingReset}
      <section class="confirm-reset" aria-label="Confirm reset">
        <p>Reset all testing progress?</p>
        <div>
          <button type="button" onclick={confirmReset}>Confirm</button>
          <button type="button" onclick={cancelReset}>Cancel</button>
        </div>
      </section>
    {/if}

    {#if resetMessage}
      <p class="reset-message">{resetMessage}</p>
    {/if}
  </section>
</main>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    width: 100%;
    min-width: 0;
    min-height: 0;
    font-family:
      Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI",
      sans-serif;
    color: #f4f0e8;
    background: transparent;
    user-select: none;
  }

  :global(body) {
    overflow: hidden;
  }

  :global(button) {
    user-select: none;
  }

  main {
    width: fit-content;
    min-height: 0;
    padding: 0;
    box-sizing: border-box;
    background: transparent;
  }

  .panel {
    width: 260px;
    max-height: 560px;
    overflow: auto;
    border: 1px solid rgba(244, 240, 232, 0.14);
    border-radius: 8px;
    padding: 10px;
    box-sizing: border-box;
    background: rgba(18, 18, 18, 0.72);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.28);
    -webkit-backdrop-filter: blur(10px);
    backdrop-filter: blur(10px);
    scrollbar-color: rgba(200, 192, 178, 0.42) rgba(18, 18, 18, 0.28);
    scrollbar-width: thin;
  }

  .mode-labels {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-bottom: 7px;
    pointer-events: none;
  }

  .mode-labels span {
    border: 1px solid rgba(244, 240, 232, 0.14);
    border-radius: 4px;
    padding: 2px 5px;
    color: #c8c0b2;
    background: rgba(24, 24, 24, 0.74);
    font-size: 0.58rem;
    font-weight: 900;
    letter-spacing: 0;
  }

  .mode-labels .dev-mode-label {
    border-color: rgba(255, 217, 87, 0.52);
    color: #241f18;
    background: #ffd957;
  }

  .panel::-webkit-scrollbar {
    width: 8px;
  }

  .panel::-webkit-scrollbar-track {
    background: rgba(18, 18, 18, 0.28);
    border-radius: 999px;
  }

  .panel::-webkit-scrollbar-thumb {
    border: 2px solid rgba(18, 18, 18, 0.28);
    border-radius: 999px;
    background: rgba(200, 192, 178, 0.42);
  }

  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 28px;
  }

  .titlebar p {
    margin: 0;
    color: #aaa296;
    font-size: 0.72rem;
    font-weight: 800;
  }

  .title-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .notification-button,
  .exit-button {
    min-width: 46px;
    min-height: 26px;
    border: 1px solid rgba(244, 240, 232, 0.13);
    border-radius: 6px;
    color: #f4f0e8;
    background: rgba(33, 24, 25, 0.88);
    font: inherit;
    font-size: 0.72rem;
    font-weight: 800;
    cursor: pointer;
  }

  .notification-button {
    min-width: 30px;
    border-color: rgba(255, 218, 87, 0.78);
    color: #1f1704;
    background: #ffd957;
    box-shadow:
      0 0 0 2px rgba(255, 217, 87, 0.16),
      0 0 16px rgba(255, 217, 87, 0.42);
    animation: notification-pulse 1.4s ease-in-out infinite;
  }

  .notification-button:hover {
    background: #ffe37b;
    box-shadow:
      0 0 0 2px rgba(255, 227, 123, 0.2),
      0 0 20px rgba(255, 227, 123, 0.52);
  }

  @keyframes notification-pulse {
    0%,
    100% {
      transform: translateY(0);
    }

    50% {
      transform: translateY(-1px);
    }
  }

  .exit-button:hover {
    background: rgba(49, 34, 37, 0.92);
  }

  .counter {
    padding: 10px 0 8px;
  }

  .label {
    margin: 0 0 4px;
    color: #b8b2a6;
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0;
    text-transform: uppercase;
  }

  .influence {
    margin: 0;
    font-size: 2.35rem;
    line-height: 1;
    font-weight: 800;
  }

  .influence-row {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 38px;
  }

  .power-proc-dot {
    display: inline-grid;
    place-items: center;
    min-width: 30px;
    height: 22px;
    border: 1px solid rgba(255, 91, 91, 0.58);
    border-radius: 999px;
    padding: 0 7px;
    color: #ffe7e7;
    background:
      radial-gradient(circle at 30% 28%, rgba(255, 215, 215, 0.72), transparent 28%),
      rgba(138, 24, 32, 0.9);
    box-shadow:
      0 0 0 2px rgba(255, 91, 91, 0.14),
      0 0 18px rgba(255, 67, 82, 0.4);
    font-size: 0.72rem;
    font-weight: 900;
    animation: power-proc-pop 1.6s ease both;
  }

  @keyframes power-proc-pop {
    0% {
      opacity: 0;
      transform: scale(0.72) translateY(3px);
    }

    16% {
      opacity: 1;
      transform: scale(1.06) translateY(0);
    }

    70% {
      opacity: 1;
      transform: scale(1) translateY(0);
    }

    100% {
      opacity: 0;
      transform: scale(0.88) translateY(-2px);
    }
  }

  .title-rank {
    position: relative;
    display: grid;
    margin-top: 7px;
    width: fit-content;
    max-width: 100%;
  }

  .title-rank p {
    margin: 0;
    color: #f4f0e8;
    font-size: 0.86rem;
    font-weight: 900;
  }

  .title-tooltip {
    position: absolute;
    left: 0;
    top: calc(100% + 6px);
    z-index: 3;
    width: max-content;
    max-width: 220px;
    border: 1px solid rgba(215, 201, 161, 0.32);
    border-radius: 6px;
    padding: 7px 8px;
    color: #f4f0e8;
    background: rgba(45, 38, 28, 0.96);
    box-shadow: 0 10px 22px rgba(0, 0, 0, 0.32);
    font-size: 0.72rem;
    font-weight: 700;
    line-height: 1.25;
    opacity: 0;
    pointer-events: none;
    transform: translateY(-2px);
    transition:
      opacity 120ms ease,
      transform 120ms ease;
  }

  .title-rank:hover .title-tooltip {
    opacity: 1;
    transform: translateY(0);
  }

  .progress-summary {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 7px;
    color: #d7c9a1;
    font-size: 0.75rem;
    font-weight: 800;
  }

  .level-progress {
    position: relative;
    display: grid;
  }

  .level-meter {
    width: 100%;
    height: 5px;
    margin-top: 7px;
    overflow: hidden;
    border-radius: 999px;
    background: rgba(244, 240, 232, 0.13);
  }

  .level-meter span {
    display: block;
    height: 100%;
    border-radius: inherit;
    background: #d7c9a1;
  }

  .xp-tooltip {
    position: absolute;
    left: 0;
    top: calc(100% + 7px);
    z-index: 3;
    width: max-content;
    max-width: 220px;
    border: 1px solid rgba(215, 201, 161, 0.32);
    border-radius: 6px;
    padding: 7px 8px;
    color: #f4f0e8;
    background: rgba(45, 38, 28, 0.96);
    box-shadow: 0 10px 22px rgba(0, 0, 0, 0.32);
    font-size: 0.72rem;
    font-weight: 700;
    line-height: 1.25;
    opacity: 0;
    pointer-events: none;
    transform: translateY(-2px);
    transition:
      opacity 120ms ease,
      transform 120ms ease;
  }

  .level-progress:hover .xp-tooltip {
    opacity: 1;
    transform: translateY(0);
  }

  .notification-popup {
    display: grid;
    gap: 8px;
    margin-bottom: 10px;
    border: 1px solid rgba(215, 201, 161, 0.32);
    border-radius: 6px;
    padding: 9px;
    background: rgba(45, 38, 28, 0.92);
  }

  .notification-popup p {
    margin: 0;
    color: #f4f0e8;
    font-size: 0.78rem;
    font-weight: 700;
  }

  .notification-popup .notification-title {
    color: #d7c9a1;
    font-size: 0.86rem;
    font-weight: 900;
  }

  .notification-popup button {
    min-height: 30px;
    border: 1px solid rgba(244, 240, 232, 0.14);
    border-radius: 6px;
    color: #f4f0e8;
    background: rgba(24, 24, 24, 0.84);
    font: inherit;
    font-size: 0.75rem;
    font-weight: 800;
    cursor: pointer;
  }

  .notification-popup button:hover {
    background: rgba(48, 43, 37, 0.94);
  }

  .actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .actions button,
  .reset-toggle {
    min-height: 34px;
    border: 1px solid rgba(244, 240, 232, 0.14);
    border-radius: 6px;
    color: #f4f0e8;
    background: rgba(36, 33, 29, 0.86);
    font: inherit;
    font-size: 0.78rem;
    font-weight: 800;
    cursor: pointer;
  }

  .actions button:hover,
  .reset-toggle:hover:not(:disabled) {
    background: rgba(48, 43, 37, 0.92);
  }

  .actions button.active-menu {
    border-color: rgba(215, 201, 161, 0.46);
    color: #1f1a12;
    background: #d7c9a1;
    box-shadow: 0 0 0 2px rgba(215, 201, 161, 0.13);
  }

  .actions button.active-menu:hover {
    background: #e3d6b3;
  }

  .details {
    margin-top: 10px;
  }

  .dev-tools {
    display: grid;
    gap: 8px;
    margin-top: 10px;
    border: 1px dashed rgba(255, 217, 87, 0.36);
    border-radius: 6px;
    padding: 8px;
    background: rgba(51, 41, 17, 0.62);
  }

  .dev-tools button {
    min-height: 30px;
    border: 1px solid rgba(255, 217, 87, 0.26);
    border-radius: 6px;
    color: #f4f0e8;
    background: rgba(36, 33, 29, 0.86);
    font: inherit;
    font-size: 0.74rem;
    font-weight: 800;
    cursor: pointer;
  }

  .dev-tools button:hover {
    background: rgba(54, 45, 24, 0.94);
  }

  .dev-tools p {
    margin: 0;
    color: #d7c9a1;
    font-size: 0.72rem;
    font-weight: 700;
  }

  dl {
    display: grid;
    gap: 6px;
    margin: 0;
  }

  dl div {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    border-top: 1px solid rgba(244, 240, 232, 0.12);
    padding-top: 6px;
  }

  dt {
    color: #c8c0b2;
    font-size: 0.8rem;
    font-weight: 700;
  }

  dd {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 800;
  }

  .shop,
  .inventory,
  .settings {
    width: 100%;
    margin-top: 10px;
  }

  .inventory,
  .settings {
    display: grid;
    gap: 10px;
  }

  .inventory-category {
    display: grid;
    gap: 7px;
    border: 1px solid rgba(244, 240, 232, 0.12);
    border-radius: 6px;
    padding: 9px;
    background: rgba(30, 28, 25, 0.82);
  }

  .inventory-heading {
    margin: 0;
    color: #d7c9a1;
    font-size: 0.8rem;
    font-weight: 900;
  }

  .inventory-category li {
    display: grid;
    gap: 3px;
    border: 0;
    border-top: 1px solid rgba(244, 240, 232, 0.12);
    border-radius: 0;
    padding-top: 7px;
    background: transparent;
  }

  .inventory-category li:first-child {
    border-top: 0;
    padding-top: 0;
  }

  .inventory-category li p {
    margin: 0;
    color: #f4f0e8;
    font-size: 0.8rem;
    font-weight: 900;
  }

  .inventory-category li small {
    color: #aaa296;
    font-size: 0.7rem;
    font-weight: 700;
    line-height: 1.25;
  }

  .empty-panel-message {
    margin: 0;
    color: #8f887d;
    font-size: 0.74rem;
    font-weight: 700;
  }

  .settings-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    border: 1px solid rgba(244, 240, 232, 0.12);
    border-radius: 6px;
    padding: 9px;
    background: rgba(30, 28, 25, 0.82);
    cursor: pointer;
  }

  .settings-toggle span {
    display: grid;
    gap: 3px;
    min-width: 0;
  }

  .settings-toggle strong {
    color: #f4f0e8;
    font-size: 0.8rem;
    font-weight: 900;
  }

  .settings-toggle small {
    color: #aaa296;
    font-size: 0.7rem;
    font-weight: 700;
    line-height: 1.25;
  }

  .settings-toggle input {
    flex: 0 0 auto;
    width: 34px;
    height: 20px;
    margin: 0;
    appearance: none;
    border: 1px solid rgba(244, 240, 232, 0.16);
    border-radius: 999px;
    background: rgba(16, 16, 16, 0.86);
    cursor: pointer;
    transition:
      border-color 120ms ease,
      background 120ms ease;
  }

  .settings-toggle input::before {
    display: block;
    width: 14px;
    height: 14px;
    margin: 2px;
    border-radius: 999px;
    background: #8f887d;
    content: "";
    transition:
      transform 120ms ease,
      background 120ms ease;
  }

  .settings-toggle input:checked {
    border-color: rgba(215, 201, 161, 0.46);
    background: rgba(215, 201, 161, 0.22);
  }

  .settings-toggle input:checked::before {
    background: #d7c9a1;
    transform: translateX(14px);
  }

  .settings-toggle input:disabled {
    cursor: default;
    opacity: 0.62;
  }

  .settings-message {
    margin: 0;
    color: #e0b2b8;
    font-size: 0.75rem;
    font-weight: 700;
  }

  ul {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    border: 1px solid rgba(244, 240, 232, 0.12);
    border-radius: 6px;
    background: rgba(30, 28, 25, 0.82);
  }

  .shop-item {
    display: grid;
    width: 100%;
    gap: 4px;
    border: 0;
    border-radius: 6px;
    padding: 9px;
    color: inherit;
    background: transparent;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .shop-item:hover:not(:disabled) {
    background: rgba(41, 37, 31, 0.92);
  }

  .shop-item:disabled {
    cursor: default;
    opacity: 0.72;
  }

  .shop-item span:first-child {
    font-size: 0.82rem;
    font-weight: 800;
  }

  .shop-item span:last-child {
    color: #c8c0b2;
    font-size: 0.75rem;
    font-weight: 700;
  }

  .shop-item small {
    color: #aaa296;
    font-size: 0.7rem;
    font-weight: 700;
    line-height: 1.25;
  }

  .confirm-purchase {
    display: grid;
    gap: 8px;
    border-top: 1px solid rgba(244, 240, 232, 0.12);
    padding: 9px;
    background: rgba(36, 33, 29, 0.86);
  }

  .confirm-purchase p {
    margin: 0;
    color: #d7c9a1;
    font-size: 0.78rem;
    font-weight: 800;
  }

  .confirm-purchase div {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .confirm-purchase button {
    min-height: 30px;
    border: 1px solid rgba(244, 240, 232, 0.14);
    border-radius: 6px;
    color: #f4f0e8;
    background: rgba(24, 24, 24, 0.84);
    font: inherit;
    font-size: 0.75rem;
    font-weight: 800;
    cursor: pointer;
  }

  .confirm-purchase button:hover {
    background: rgba(48, 43, 37, 0.94);
  }

  .purchase-message {
    margin: 10px 0 0;
    color: #d7c9a1;
    font-size: 0.75rem;
    font-weight: 700;
  }

  .reset-toggle {
    width: 100%;
    margin-top: 8px;
    background: rgba(33, 24, 25, 0.84);
  }

  .reset-toggle:disabled {
    cursor: default;
    opacity: 0.72;
  }

  .confirm-reset {
    display: grid;
    gap: 8px;
    width: 100%;
    margin-top: 8px;
    border: 1px solid rgba(224, 178, 184, 0.22);
    border-radius: 6px;
    padding: 9px;
    box-sizing: border-box;
    background: rgba(33, 24, 25, 0.86);
  }

  .confirm-reset p {
    margin: 0;
    color: #e0b2b8;
    font-size: 0.78rem;
    font-weight: 800;
  }

  .confirm-reset div {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .confirm-reset button {
    min-height: 30px;
    border: 1px solid rgba(224, 178, 184, 0.24);
    border-radius: 6px;
    color: #f4f0e8;
    background: rgba(24, 24, 24, 0.84);
    font: inherit;
    font-size: 0.75rem;
    font-weight: 800;
    cursor: pointer;
  }

  .confirm-reset button:hover {
    background: rgba(48, 35, 38, 0.94);
  }

  .reset-message {
    margin: 8px 0 0;
    color: #e0b2b8;
    font-size: 0.75rem;
    font-weight: 700;
  }
</style>
