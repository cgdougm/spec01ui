import { soundFor } from "./sound.js";
const patterns = {
  selection: 8,
  light: 12,
  rigid: 14,
  soft: 18,
  medium: 22,
  heavy: 36,
  success: [14, 70, 22],
  warning: [24, 90, 24],
  error: [28, 60, 28, 60, 28]
};
const canVibrate = () => typeof navigator !== "undefined" && typeof navigator.vibrate === "function";
const hasSwitch = () => typeof HTMLInputElement !== "undefined" && "switch" in HTMLInputElement.prototype;
let label = null;
function tapSwitch() {
  if (!label) {
    label = document.createElement("label");
    label.dataset.hapticSwitch = "";
    label.inert = true;
    label.setAttribute("aria-hidden", "true");
    label.style.cssText = "position:fixed;top:0;left:0;width:1px;height:1px;overflow:hidden;opacity:0.01;pointer-events:none";
    const input = document.createElement("input");
    input.type = "checkbox";
    input.setAttribute("switch", "");
    input.tabIndex = -1;
    label.append(input);
    document.body.append(label);
  }
  label.click();
}
let last = 0;
function haptic(kind) {
  if (typeof document === "undefined") return;
  const now = performance.now();
  if (now - last < 30) return;
  last = now;
  soundFor(kind);
  if (canVibrate()) {
    navigator.vibrate(patterns[kind]);
  } else if (hasSwitch()) {
    if (navigator.userActivation && !navigator.userActivation.isActive) return;
    tapSwitch();
  }
}
const crosses = (from, to, size) => Math.floor(from / size) !== Math.floor(to / size);
const latch = (el) => {
  const box = el.matches('[data-kit="checkbox"]') ? el : el.querySelector('[data-kit="checkbox"]');
  return box?.hasAttribute("data-checked") ? "soft" : "rigid";
};
let pressed = { x: 0, y: 0 };
let moved = false;
const presses = [
  ['[data-kit="check"], [data-kit="checkbox"]', latch],
  ['[data-kit="switch"]', "medium"],
  ['[data-kit="dpad-zone"], [data-kit="rocker-part"]', "light"],
  ['[data-kit="jog-center"]', "medium"],
  // A tap on a slider's track or the page wheel jumps the value. Their drags
  // tick from their own handlers, and a drag's closing click isn't a tap.
  // Where the jump already ticked on the press (Android; iOS won't play a
  // haptic before the click), the click adds nothing.
  ['[data-kit="slider-control"], [data-kit="page-wheel"]', () => moved || performance.now() - last < 200 ? "none" : "selection"],
  [
    [
      '[data-kit="segment"]',
      '[data-kit="tab"]',
      '[data-kit="radio"]',
      '[data-kit="slide-switch-stop"]',
      '[data-kit="mode-stop"]',
      '[data-kit="toggle"]',
      '[data-kit="page"]',
      '[data-kit="key-swatch"]',
      '[data-kit="item-list-row"]',
      '[data-kit="accordion-trigger"]',
      '[data-kit="menubar-trigger"]',
      '[data-kit="carousel-indicator"]',
      '[data-kit="navbar-link"]',
      '[role="menuitem"]',
      '[role="menuitemcheckbox"]',
      '[role="option"]',
      '[role="gridcell"]'
    ].join(", "),
    "selection"
  ],
  ['[data-kit="button"], [data-kit="code-copy"]', (el) => el.matches('[data-tone="primary"]') ? "medium" : "light"]
];
function installHaptics() {
  document.addEventListener(
    "pointerdown",
    (event) => {
      pressed = { x: event.clientX, y: event.clientY };
    },
    true
  );
  document.addEventListener(
    "click",
    (event) => {
      const target = event.target;
      if (!target?.closest) return;
      if (target.closest("[data-haptic-switch]")) return;
      moved = Math.hypot(event.clientX - pressed.x, event.clientY - pressed.y) > 6;
      for (const [selector, kind] of presses) {
        const el = target.closest(selector);
        if (!el) continue;
        if (el.matches(':disabled, [aria-disabled="true"], [data-disabled]')) return;
        const tap = typeof kind === "function" ? kind(el) : kind;
        if (tap !== "none") haptic(tap);
        return;
      }
    },
    true
  );
}
export {
  crosses,
  haptic,
  installHaptics
};
