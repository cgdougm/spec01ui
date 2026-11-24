const storageKey = "spec01:sound";
let enabled = typeof localStorage === "undefined" || localStorage.getItem(storageKey) !== "off";
const listeners = /* @__PURE__ */ new Set();
function soundEnabled() {
  return enabled;
}
function setSoundEnabled(on) {
  enabled = on;
  try {
    localStorage.setItem(storageKey, on ? "on" : "off");
  } catch {
  }
  listeners.forEach((listener) => listener(on));
}
function onSoundChange(listener) {
  listeners.add(listener);
  return () => void listeners.delete(listener);
}
let ctx = null;
let master = null;
let noise = null;
function audio() {
  if (ctx?.state === "closed") ctx = null;
  if (!ctx) {
    const Context = window.AudioContext ?? window.webkitAudioContext;
    if (!Context) return null;
    ctx = new Context({ latencyHint: "interactive" });
    master = ctx.createGain();
    master.gain.value = 0.55;
    master.connect(ctx.destination);
    noise = ctx.createBuffer(1, ctx.sampleRate / 2, ctx.sampleRate);
    const data = noise.getChannelData(0);
    for (let i = 0; i < data.length; i++) data[i] = Math.random() * 2 - 1;
  }
  if (ctx.state !== "running") ctx.resume().catch(() => {
  });
  return ctx;
}
function unlock() {
  if (enabled && typeof window !== "undefined") audio();
}
const vary = (amount) => 1 + (Math.random() * 2 - 1) * amount;
function burst(at, { type = "bandpass", freq, q = 1, level, decay }) {
  const c = ctx;
  const source = c.createBufferSource();
  source.buffer = noise;
  source.playbackRate.value = vary(0.1);
  const filter = c.createBiquadFilter();
  filter.type = type;
  filter.frequency.value = freq * vary(0.06);
  filter.Q.value = q;
  const gain = c.createGain();
  gain.gain.setValueAtTime(0, at);
  gain.gain.linearRampToValueAtTime(level * vary(0.15), at + 8e-4);
  gain.gain.exponentialRampToValueAtTime(1e-4, at + decay);
  source.connect(filter).connect(gain).connect(master);
  source.start(at, Math.random() * 0.4);
  source.stop(at + decay + 0.02);
}
function tone(at, { type = "sine", freq, to = freq, level, decay, attack = 1e-3 }) {
  const c = ctx;
  const osc = c.createOscillator();
  osc.type = type;
  const f = freq * vary(0.04);
  osc.frequency.setValueAtTime(f, at);
  osc.frequency.exponentialRampToValueAtTime(Math.max(20, to / freq * f), at + decay);
  const gain = c.createGain();
  gain.gain.setValueAtTime(0, at);
  gain.gain.linearRampToValueAtTime(level * vary(0.12), at + attack);
  gain.gain.exponentialRampToValueAtTime(1e-4, at + decay);
  osc.connect(gain).connect(master);
  osc.start(at);
  osc.stop(at + decay + 0.02);
}
const voices = {
  // A key bottoming out: a click on top of a short, falling body.
  down: (at) => {
    burst(at, { freq: 2600, q: 0.9, level: 0.32, decay: 0.022 });
    tone(at, { freq: 210, to: 130, level: 0.4, decay: 0.05 });
  },
  // A primary key is bigger and lower.
  "down-primary": (at) => {
    burst(at, { freq: 2100, q: 0.8, level: 0.36, decay: 0.026 });
    tone(at, { freq: 165, to: 100, level: 0.55, decay: 0.07 });
  },
  // The key returning: lighter and higher.
  up: (at) => {
    burst(at, { freq: 3900, q: 1.2, level: 0.16, decay: 0.012 });
    tone(at, { freq: 300, to: 240, level: 0.1, decay: 0.018 });
  },
  // A latch catching: a soft, low clack with a faint spring behind it.
  "latch-down": (at) => {
    burst(at, { freq: 2e3, q: 0.8, level: 0.18, decay: 0.018 });
    tone(at, { freq: 160, to: 110, level: 0.2, decay: 0.045 });
    burst(at + 4e-3, { freq: 1150, q: 14, level: 0.035, decay: 0.05 });
  },
  // A latch releasing: a light snap and a fainter, brighter ring.
  "latch-up": (at) => {
    burst(at, { freq: 3200, q: 1, level: 0.11, decay: 0.012 });
    burst(at + 3e-3, { freq: 1600, q: 12, level: 0.03, decay: 0.04 });
  },
  // A detent: a tiny, bright click.
  tick: (at) => {
    burst(at, { type: "highpass", freq: 3200, q: 0.7, level: 0.2, decay: 6e-3 });
    tone(at, { freq: 4300, level: 0.035, decay: 0.01 });
  },
  // A piezo beep: a soft square tone through a low-pass.
  blip: (at) => {
    tone(at, { type: "square", freq: 2350, level: 0.035, decay: 0.045, attack: 2e-3 });
  },
  // A keyboard switch: crisper and shorter than a device key.
  type: (at) => {
    burst(at, { freq: 3300, q: 1.1, level: 0.22, decay: 0.014 });
    tone(at, { freq: 260, to: 190, level: 0.18, decay: 0.03 });
  },
  // The space bar: a longer, lower clack with a little rattle.
  "type-space": (at) => {
    burst(at, { freq: 2200, q: 0.8, level: 0.26, decay: 0.022 });
    tone(at, { freq: 170, to: 120, level: 0.26, decay: 0.05 });
    burst(at + 6e-3, { freq: 900, q: 6, level: 0.05, decay: 0.03 });
  },
  // Enter and Backspace: bigger keys, a heavier thock.
  "type-big": (at) => {
    burst(at, { freq: 2600, q: 0.9, level: 0.26, decay: 0.018 });
    tone(at, { freq: 200, to: 140, level: 0.28, decay: 0.045 });
  },
  // A key coming back up.
  "type-up": (at) => {
    burst(at, { freq: 4400, q: 1.3, level: 0.08, decay: 8e-3 });
  },
  // A popup flipping open: a light tap and a short rising note.
  open: (at) => {
    burst(at, { freq: 2600, q: 1, level: 0.1, decay: 0.012 });
    tone(at, { freq: 520, to: 820, level: 0.045, decay: 0.06, attack: 3e-3 });
  },
  success: (at) => {
    tone(at, { freq: 1318, level: 0.07, decay: 0.09, attack: 4e-3 });
    tone(at + 0.075, { freq: 1760, level: 0.07, decay: 0.16, attack: 4e-3 });
  },
  warning: (at) => {
    tone(at, { type: "triangle", freq: 880, level: 0.08, decay: 0.08 });
    tone(at + 0.11, { type: "triangle", freq: 880, level: 0.08, decay: 0.08 });
  },
  error: (at) => {
    tone(at, { type: "square", freq: 220, level: 0.04, decay: 0.09 });
    tone(at + 0.12, { type: "square", freq: 196, level: 0.04, decay: 0.14 });
  }
};
let lastPlayed = 0;
function play(voice) {
  if (!enabled || typeof window === "undefined") return;
  const c = audio();
  if (!c || !master) return;
  lastPlayed = performance.now();
  voices[voice](c.currentTime + 2e-3);
}
const sounding = /-(content|popup|positioner)$/;
const silent = /^(hover-card|tooltip)-/;
function watchPopups() {
  const heard = /* @__PURE__ */ new WeakSet();
  const opened = (el) => {
    const slot = el.getAttribute("data-slot") ?? "";
    if (!el.hasAttribute("data-open") || !sounding.test(slot) || silent.test(slot) || heard.has(el)) return false;
    heard.add(el);
    return true;
  };
  new MutationObserver((records) => {
    let open = false;
    for (const record of records) {
      if (record.type === "attributes") {
        const el = record.target;
        if (!el.hasAttribute("data-open")) heard.delete(el);
        else if (opened(el)) open = true;
      } else {
        for (const node of record.addedNodes) {
          if (!(node instanceof Element)) continue;
          if (opened(node)) open = true;
          for (const el of node.querySelectorAll("[data-open][data-slot]")) if (opened(el)) open = true;
        }
      }
    }
    if (open && performance.now() - lastPlayed >= 150) play("open");
  }).observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ["data-open"] });
}
const screen = '[data-kit="item-list"], [data-kit="lcd"], [data-kit="date-lcd"], [data-kit="hover-lcd"], [data-kit="display-screen"]';
let pointerTarget = null;
let pointerAt = 0;
let downKey = null;
const keyLike = '[data-kit="button"], [data-kit="code-copy"], [data-kit="check"], [data-kit="checkbox"], [data-kit="dpad-zone"], [data-kit="rocker-part"], [data-kit="jog-center"]';
function context() {
  return performance.now() - pointerAt < 1500 ? pointerTarget : document.activeElement;
}
function soundFor(kind) {
  const el = context();
  switch (kind) {
    case "selection":
      return play(el?.closest(screen) ? "blip" : "tick");
    case "light":
    case "medium":
    case "heavy": {
      const key = el?.closest(keyLike);
      if (key && key === downKey) {
        downKey = null;
        return play("up");
      }
      return play(kind === "light" ? "down" : "down-primary");
    }
    case "rigid":
    case "soft": {
      const key = el?.closest(keyLike);
      if (key && key === downKey) {
        downKey = null;
        return kind === "rigid" ? void 0 : play("latch-up");
      }
      return play(kind === "rigid" ? "latch-down" : "latch-up");
    }
    default:
      return play(kind);
  }
}
const typing = 'textarea, [contenteditable="true"], input:not([type]), input[type="text"], input[type="search"], input[type="email"], input[type="url"], input[type="tel"], input[type="password"], input[type="number"]';
function keyVoice(key) {
  if (key === " ") return "type-space";
  if (key === "Enter" || key === "Backspace" || key === "Delete" || key === "Tab") return "type-big";
  if (key.length === 1 || key === "Unidentified" || key === "Process") return "type";
  return null;
}
function installSound() {
  for (const type of ["pointerdown", "pointerup", "touchend", "keydown", "click"])
    document.addEventListener(type, unlock, { capture: true, passive: true });
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible" && ctx && ctx.state !== "running") ctx.resume().catch(() => {
    });
  });
  watchPopups();
  document.addEventListener(
    "keydown",
    (event) => {
      if (event.isComposing || event.metaKey || event.ctrlKey || event.altKey) return;
      if (!event.target?.closest?.(typing)) return;
      const voice = keyVoice(event.key);
      if (!voice) return;
      play(event.repeat ? "tick" : voice);
    },
    true
  );
  document.addEventListener(
    "keyup",
    (event) => {
      if (event.isComposing || event.metaKey || event.ctrlKey || event.altKey) return;
      if (!event.target?.closest?.(typing)) return;
      if (keyVoice(event.key)) play("type-up");
    },
    true
  );
  document.addEventListener(
    "pointerdown",
    (event) => {
      pointerTarget = event.target;
      pointerAt = performance.now();
      if (event.button !== 0) return;
      const key = event.target?.closest?.(keyLike);
      if (!key || key.matches(':disabled, [aria-disabled="true"], [data-disabled]')) return;
      downKey = key;
      const latch = key.matches('[data-kit="check"], [data-kit="checkbox"]') || key.querySelector('[data-kit="checkbox"]');
      if (latch) {
        const box = key.matches('[data-kit="checkbox"]') ? key : key.querySelector('[data-kit="checkbox"]');
        play(box?.hasAttribute("data-checked") ? "up" : "latch-down");
      } else {
        play(key.matches('[data-tone="primary"]') ? "down-primary" : "down");
      }
    },
    true
  );
}
export {
  installSound,
  onSoundChange,
  setSoundEnabled,
  soundEnabled,
  soundFor
};
