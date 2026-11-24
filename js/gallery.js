// Vanilla behaviors for the SPEC-01 gallery.
//
// The skin reads all state from attributes, so each control here only flips
// the same attributes ui-sdk would: data-checked / data-unchecked, data-pressed,
// data-selected, data-open / data-closed, aria-*, and a few CSS variables
// (--at, --i, --turn, --lit, slider and indicator positions). Haptics and
// sounds come from SPEC-01's own page-wide listeners (installHardware), which
// react to clicks on [data-kit] parts whatever renders them.

import { installHardware } from "../vendor/spec-01/lib/install.js"
import { haptic, crosses } from "../vendor/spec-01/lib/haptics.js"
import { soundEnabled, setSoundEnabled } from "../vendor/spec-01/lib/sound.js"

installHardware()

const root = document.querySelector('[data-skin="hardware"]')
const $$ = (sel, el = document) => [...el.querySelectorAll(sel)]
const live = (el) => el.closest('[data-kit="module"]')?.setAttribute("data-gallery-live", "")

// ---------------------------------------------------------------- state helpers

function setChecked(el, on) {
  el.setAttribute("aria-checked", String(on))
  el.toggleAttribute("data-checked", on)
  el.toggleAttribute("data-unchecked", !on)
  for (const part of el.querySelectorAll("[data-slot]")) {
    part.toggleAttribute("data-checked", on)
    part.toggleAttribute("data-unchecked", !on)
  }
}

function setPressed(el, on) {
  el.setAttribute("aria-pressed", String(on))
  el.toggleAttribute("data-pressed", on)
}

function setOpen(el, open) {
  el.toggleAttribute("data-open", open)
  el.toggleAttribute("data-closed", !open)
}

// Roving tabindex, as ui-sdk does in radio groups, tabs and toolbars.
function rove(items, current) {
  for (const item of items) item.tabIndex = item === current ? 0 : -1
}

// Arrow keys move through a group and activate, as radios and tabs do.
function arrowKeys(container, items, activate) {
  container.addEventListener("keydown", (event) => {
    const forward = ["ArrowRight", "ArrowDown"].includes(event.key)
    const back = ["ArrowLeft", "ArrowUp"].includes(event.key)
    if (!forward && !back) return
    const list = items()
    const i = list.indexOf(document.activeElement)
    if (i < 0) return
    event.preventDefault()
    const next = list[(i + (forward ? 1 : -1) + list.length) % list.length]
    next.focus()
    activate(next)
    haptic("selection")
  })
}

// ---------------------------------------------------------------- key color

const picker = document.querySelector('[data-kit="key-picker"]')
const swatches = $$('[data-kit="key-swatch"]', picker)
function pickKey(swatch) {
  const [, l, c, h] = swatch.style.getPropertyValue("--swatch").match(/oklch\(([\d.]+) ([\d.]+) ([\d.]+)\)/)
  root.style.setProperty("--theme-l", l)
  root.style.setProperty("--theme-c", c)
  root.style.setProperty("--theme-h", h)
  root.dataset.key = swatch.title.toLowerCase()
  for (const s of swatches) setChecked(s, s === swatch)
  rove(swatches, swatch)
  try { localStorage.setItem("spec01ui:key", swatch.title) } catch {}
}
picker.addEventListener("click", (e) => {
  const s = e.target.closest('[data-kit="key-swatch"]')
  if (s) pickKey(s)
})
arrowKeys(picker, () => swatches, pickKey)
try {
  const saved = swatches.find((s) => s.title === localStorage.getItem("spec01ui:key"))
  if (saved) pickKey(saved)
} catch {}

// ---------------------------------------------------------------- sound toggle

const soundToggle = document.querySelector('[data-kit="sound-toggle"]')
function showSound() {
  const on = soundEnabled()
  soundToggle.setAttribute("aria-pressed", String(on))
  soundToggle.title = on ? "Sounds on" : "Sounds off"
}
soundToggle.addEventListener("click", () => {
  setSoundEnabled(!soundEnabled())
  showSound()
})
showSound()

// ---------------------------------------------------------------- switch & checkbox

for (const sw of $$('[data-kit="switch"], [data-kit="checkbox"]')) {
  live(sw)
  sw.addEventListener("click", () => setChecked(sw, sw.getAttribute("aria-checked") !== "true"))
}

// ---------------------------------------------------------------- radio groups
// Segmented control, radio group, mode dial, slide switch: one checked radio,
// plus whatever CSS variable each one draws from.

const radioGroups = $$('[role="radiogroup"]').filter((g) => g !== picker)
for (const group of radioGroups) {
  const radios = () => $$('[role="radio"]', group)
  const select = (radio) => {
    const list = radios()
    for (const r of list) setChecked(r, r === radio)
    rove(list, radio)
    const kit = group.dataset.kit
    if (kit === "mode-dial") group.style.setProperty("--at", radio.style.getPropertyValue("--a"))
    if (kit === "slide-switch") group.style.setProperty("--i", list.indexOf(radio))
    if (kit === "segmented") placeIndicator(group.querySelector('[data-kit="segmented-indicator"]'), radio, "segmented-control-indicator")
  }
  live(group)
  group.addEventListener("click", (e) => {
    const r = e.target.closest('[role="radio"]')
    if (r && group.contains(r)) select(r)
  })
  arrowKeys(group, radios, select)
}

// Indicators that slide under the chosen item read its box from variables.
function placeIndicator(indicator, item, prefix) {
  if (!indicator) return
  const box = indicator.offsetParent ?? indicator.parentElement
  const left = item.offsetLeft
  const top = item.offsetTop
  indicator.style.setProperty(`--${prefix}-left`, `${left}px`)
  indicator.style.setProperty(`--${prefix}-top`, `${top}px`)
  indicator.style.setProperty(`--${prefix}-width`, `${item.offsetWidth}px`)
  indicator.style.setProperty(`--${prefix}-height`, `${item.offsetHeight}px`)
  indicator.style.setProperty(`--${prefix}-right`, `${box.clientWidth - left - item.offsetWidth}px`)
  indicator.style.setProperty(`--${prefix}-bottom`, `${box.clientHeight - top - item.offsetHeight}px`)
}

// ---------------------------------------------------------------- toggles
// In a toggle group, one at a time; on a toolbar, each on its own.

for (const toggle of $$('[data-kit="toggle"]')) {
  live(toggle)
  toggle.addEventListener("click", () => {
    const group = toggle.closest('[data-slot="toggle-group"]')
    if (group) {
      for (const t of $$('[data-kit="toggle"]', group)) setPressed(t, t === toggle)
    } else {
      setPressed(toggle, toggle.getAttribute("aria-pressed") !== "true")
    }
  })
}

// ---------------------------------------------------------------- tabs

const tabBodies = {
  mix: "Levels, pan and sends for every track on the current pattern.",
  edit: "Trim, loop and reverse the selected sample. Changes are non-destructive.",
  fx: "Two effect slots per track: punch-in filters, delay and tape stop.",
}
for (const tabs of $$('[data-kit="tabs"]')) {
  const list = tabs.querySelector('[role="tablist"]')
  const panel = tabs.querySelector('[role="tabpanel"]')
  const indicator = tabs.querySelector('[data-kit="tabs-indicator"]')
  const all = () => $$('[role="tab"]', list)
  const select = (tab) => {
    for (const t of all()) {
      const on = t === tab
      t.setAttribute("aria-selected", String(on))
      t.toggleAttribute("data-selected", on)
    }
    rove(all(), tab)
    placeIndicator(indicator, tab, "active-tab")
    const key = tab.id.split("-tab-").pop()
    if (panel && tabBodies[key]) {
      panel.textContent = tabBodies[key]
      panel.setAttribute("aria-labelledby", tab.id)
    }
  }
  live(tabs)
  list.addEventListener("click", (e) => {
    const t = e.target.closest('[role="tab"]')
    if (t) select(t)
  })
  arrowKeys(list, all, select)
}

// ---------------------------------------------------------------- accordion

const accordionBodies = [
  "64 GB internal. Projects back up to your computer over USB-C.",
  "About 16 hours of playback. Charges to 80% in an hour.",
  "New features arrive as free updates, installed from the device.",
]
for (const accordion of $$('[data-kit="accordion"]')) {
  live(accordion)
  accordion.addEventListener("click", (e) => {
    const trigger = e.target.closest('[data-kit="accordion-trigger"]')
    if (!trigger) return
    const item = trigger.closest('[data-kit="accordion-item"]')
    const open = trigger.getAttribute("aria-expanded") !== "true"
    // One open at a time, as the demo is set up.
    for (const other of $$('[data-kit="accordion-item"]', accordion)) {
      const on = other === item && open
      const t = other.querySelector('[data-kit="accordion-trigger"]')
      t.setAttribute("aria-expanded", String(on))
      for (const el of [other, other.querySelector('[data-kit="accordion-header"]'), t]) setOpen(el, on)
      let panel = other.querySelector('[data-kit="accordion-panel"]')
      if (on && !panel) {
        panel = document.createElement("div")
        panel.dataset.kit = "accordion-panel"
        panel.dataset.slot = "accordion-panel"
        panel.setAttribute("role", "region")
        panel.id = t.id.replace("-trigger-", "-panel-")
        panel.setAttribute("aria-labelledby", t.id)
        panel.style.cssText = "--accordion-panel-height: auto; --accordion-panel-width: auto;"
        panel.innerHTML = `<p>${accordionBodies[Number(other.dataset.index)] ?? ""}</p>`
        other.append(panel)
      }
      if (on) t.setAttribute("aria-controls", panel.id)
      if (!on && panel) panel.remove()
    }
  })
}

// ---------------------------------------------------------------- slider

for (const control of $$('[data-kit="slider-control"]')) {
  const slider = control.closest('[data-kit="slider"]')
  const track = control.querySelector('[data-kit="slider-track"]')
  const indicator = control.querySelector('[data-kit="slider-indicator"]')
  const thumb = control.querySelector('[data-kit="slider-thumb"]')
  const output = slider?.querySelector('[data-kit="value"]')
  let value = Number(output?.textContent ?? 50)
  thumb.tabIndex = 0
  thumb.setAttribute("role", "slider")
  thumb.setAttribute("aria-valuemin", "0")
  thumb.setAttribute("aria-valuemax", "100")

  const set = (next) => {
    next = Math.round(Math.min(100, Math.max(0, next)))
    if (next === value) return
    if (crosses(value, next, 10)) haptic("selection")
    value = next
    indicator.style.width = `${value}%`
    indicator.style.setProperty("--slider-indicator-size", `${value}%`)
    thumb.style.insetInline = `calc(${value - 50}%) calc(${50 - value}%)`
    thumb.style.setProperty("--slider-thumb-position", `${value}%`)
    thumb.setAttribute("aria-valuenow", String(value))
    if (output) output.textContent = String(value)
  }
  const fromPointer = (event) => {
    const r = track.getBoundingClientRect()
    set(((event.clientX - r.left) / r.width) * 100)
  }
  live(control)
  thumb.setAttribute("aria-valuenow", String(value))
  control.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) return
    control.setPointerCapture(event.pointerId)
    thumb.setAttribute("data-dragging", "")
    fromPointer(event)
  })
  control.addEventListener("pointermove", (event) => {
    if (control.hasPointerCapture(event.pointerId)) fromPointer(event)
  })
  control.addEventListener("pointerup", () => thumb.removeAttribute("data-dragging"))
  thumb.addEventListener("keydown", (event) => {
    const step = { ArrowRight: 1, ArrowUp: 1, ArrowLeft: -1, ArrowDown: -1, PageUp: 10, PageDown: -10 }[event.key]
    if (step === undefined) return
    event.preventDefault()
    set(value + step)
  })
}

// ---------------------------------------------------------------- flat knob
// Drag up or down on the dial, or use the arrow keys / type into the readout.
// 270 degrees of turn; 28 ticks light up to the value, as in flat-knob.tsx.

for (const knob of $$('[data-kit="knob"]')) {
  const dial = knob.querySelector('[data-kit="knob-dial"]')
  const readout = knob.querySelector('[data-kit="knob-readout"]')
  const ticks = $$('[data-kit="knob-ticks"] line', knob)
  const hidden = knob.parentElement.querySelector('input[type="hidden"]')
  let value = Number(readout.value)
  const litTicks = (v) => Math.floor((v * (ticks.length - 1)) / 100)

  const set = (next) => {
    next = Math.round(Math.min(100, Math.max(0, next)))
    if (next === value) return
    if (litTicks(value) !== litTicks(next)) haptic("selection")
    value = next
    const share = value / 100
    knob.style.setProperty("--turn", `${share * 270 - 135}deg`)
    knob.style.setProperty("--lit", share)
    readout.value = String(value)
    if (hidden) hidden.value = String(value)
    ticks.forEach((t, i) => (i <= litTicks(value) ? t.setAttribute("data-lit", "true") : t.removeAttribute("data-lit")))
  }
  live(knob)
  let drag = null
  dial.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) return
    dial.setPointerCapture(event.pointerId)
    drag = { y: event.clientY, from: value }
    knob.setAttribute("data-scrubbing", "")
  })
  dial.addEventListener("pointermove", (event) => {
    if (drag) set(drag.from + (drag.y - event.clientY) / 2)
  })
  dial.addEventListener("pointerup", () => {
    drag = null
    knob.removeAttribute("data-scrubbing")
  })
  readout.addEventListener("keydown", (event) => {
    const step = { ArrowUp: 1, ArrowDown: -1, PageUp: 10, PageDown: -10 }[event.key]
    if (step === undefined) return
    event.preventDefault()
    set(value + step)
  })
  readout.addEventListener("change", () => {
    const n = Number(readout.value)
    if (Number.isFinite(n)) set(n)
    readout.value = String(value)
  })
}

// ---------------------------------------------------------------- select wheel
// A drum of rows you scroll or drag; the row in the lens is chosen. A port
// of the paint and choose logic in select.tsx.

const wheelRow = 32
const wheelRadius = wheelRow / ((22 * Math.PI) / 180)
const ridgePitch = 4

for (const wheel of $$('[data-kit="wheel"]')) {
  const ridges = $$("[data-ridge]", wheel.parentElement)
  const items = $$('[role="option"]', wheel)
  const faces = items.map((item) => item.querySelector('[data-kit="wheel-face"]'))
  const half = ridges.length / 2 / 2 // ridges are split across two rails
  let chosen = Math.max(0, items.findIndex((i) => i.getAttribute("aria-selected") === "true"))
  let frame = 0
  let settle = 0

  const paint = () => {
    const at = wheel.scrollTop / wheelRow
    const phase = ((wheel.scrollTop % ridgePitch) + ridgePitch) % ridgePitch
    for (const ridge of ridges) {
      const angle = ((Number(ridge.dataset.ridge) - half) * ridgePitch - phase) / wheelRadius
      const visible = Math.abs(angle) < Math.PI / 2
      ridge.style.transform = visible
        ? `translateY(${(wheelRadius * Math.sin(angle)).toFixed(2)}px) scaleY(${Math.cos(angle).toFixed(3)})`
        : "scaleY(0)"
      ridge.style.opacity = visible ? (0.2 + 0.8 * Math.cos(angle)).toFixed(2) : "0"
    }
    faces.forEach((face, i) => {
      const d = i - at
      const angle = Math.max(-90, Math.min(90, d * 22))
      const rad = (angle * Math.PI) / 180
      face.style.transform = `perspective(500px) translateY(${wheelRadius * Math.sin(rad) - d * wheelRow}px) translateZ(${wheelRadius * (Math.cos(rad) - 1)}px) rotateX(${-angle}deg)`
      face.style.opacity = String(Math.max(0, 1 - Math.abs(d) * 0.28))
    })
  }
  const choose = (i) => {
    if (i === chosen) return
    chosen = i
    haptic("selection")
    items.forEach((item, n) => {
      item.setAttribute("aria-selected", String(n === i))
      item.toggleAttribute("data-selected", n === i)
    })
  }
  const centered = () => Math.max(0, Math.min(items.length - 1, Math.round(wheel.scrollTop / wheelRow)))

  // A scroll the wheel starts itself, toward a row, passes over others on
  // the way; only scrolls the person makes choose rows as they pass.
  let target = null
  const rollTo = (i, behavior = "smooth") => {
    target = i * wheelRow
    wheel.scrollTo({ top: target, behavior })
    choose(i)
  }

  live(wheel)
  target = chosen * wheelRow
  wheel.scrollTo({ top: target, behavior: "instant" }) // the skin sets scroll-behavior: smooth
  paint()
  wheel.addEventListener("scroll", () => {
    frame ||= requestAnimationFrame(() => {
      frame = 0
      paint()
    })
    if (target !== null) {
      if (Math.abs(wheel.scrollTop - target) < 1) target = null
      return
    }
    choose(centered())
    clearTimeout(settle)
    settle = setTimeout(() => {
      if (!drag) choose(centered())
    }, 120)
  })
  // Mouse drag turns the drum; touch and trackpads scroll it natively.
  let drag = null
  wheel.addEventListener("pointerdown", (e) => {
    if (e.pointerType === "mouse" && e.button === 0) drag = { y: e.clientY, top: wheel.scrollTop, moved: false }
  })
  wheel.addEventListener("pointermove", (e) => {
    if (!drag) return
    const dy = e.clientY - drag.y
    if (!drag.moved && Math.abs(dy) < 3) return
    if (!drag.moved) wheel.setPointerCapture(e.pointerId)
    drag.moved = true
    wheel.scrollTop = drag.top - dy
  })
  wheel.addEventListener("pointerup", () => {
    if (drag?.moved) rollTo(centered())
    drag = null
  })
  wheel.addEventListener("click", (e) => {
    const item = e.target.closest('[role="option"]')
    if (item) rollTo(items.indexOf(item))
  })
  wheel.addEventListener("keydown", (e) => {
    const d = { ArrowDown: 1, ArrowUp: -1 }[e.key]
    if (d === undefined) return
    e.preventDefault()
    rollTo(Math.max(0, Math.min(items.length - 1, chosen + d)))
  })
  wheel.tabIndex = 0
}

// ---------------------------------------------------------------- number stepper

for (const group of $$('[data-kit="stepper"]')) {
  const input = group.querySelector('[data-kit="stepper-input"]')
  const step = (d) => {
    const n = (Number(input.value) || 0) + d
    input.value = String(Math.min(300, Math.max(20, n)))
  }
  live(group)
  group.querySelector('[data-slot="number-input-decrement"]')?.addEventListener("click", () => step(-1))
  group.querySelector('[data-slot="number-input-increment"]')?.addEventListener("click", () => step(1))
  input.addEventListener("keydown", (event) => {
    const d = { ArrowUp: 1, ArrowDown: -1 }[event.key]
    if (d === undefined) return
    event.preventDefault()
    step(d)
  })
}

// Text inputs work as they are: mark their modules live too.
for (const el of $$('[data-kit="text-field"] input, textarea, [data-kit="input"]')) live(el)
// Plain buttons only press and click (haptics/sound come from installHardware).
for (const el of $$('#button [data-kit="button"]')) live(el)
