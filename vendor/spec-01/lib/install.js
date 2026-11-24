import { installBlockCaret } from "./block-caret.js";
import { installHaptics } from "./haptics.js";
import { installInputModality } from "./input-modality.js";
import { installSafariTabbing } from "./safari-tabbing.js";
import { installSound } from "./sound.js";
let installed = false;
function installHardware() {
  if (installed || typeof document === "undefined") return;
  installed = true;
  installBlockCaret();
  installHaptics();
  installSound();
  installSafariTabbing();
  installInputModality();
}
export {
  installHardware
};
