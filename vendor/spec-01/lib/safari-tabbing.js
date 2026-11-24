const selector = [
  "button",
  "a[href]",
  "summary",
  'input[type="range"]',
  'input[type="checkbox"]',
  'input[type="color"]',
  "select"
].map((s) => `${s}:not([tabindex])`).join(", ");
function isSafari() {
  const ua = navigator.userAgent;
  return /Safari\//.test(ua) && !/Chrome\/|Chromium\/|CriOS\/|FxiOS\/|EdgiOS\/|Android/.test(ua);
}
function mark(root) {
  if (root instanceof Element && root.matches(selector)) root.setAttribute("tabindex", "0");
  for (const el of root.querySelectorAll(selector)) el.setAttribute("tabindex", "0");
}
function installSafariTabbing() {
  if (typeof document === "undefined" || !isSafari()) return;
  mark(document);
  new MutationObserver((records) => {
    for (const record of records) {
      for (const node of record.addedNodes) if (node instanceof Element) mark(node);
    }
  }).observe(document.body, { childList: true, subtree: true });
}
export {
  installSafariTabbing
};
