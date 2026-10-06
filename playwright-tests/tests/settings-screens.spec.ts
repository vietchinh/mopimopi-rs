// Every settings page and tab, and the colour picker's states, against the original.
import { test } from "../support/fixtures";
import { openSettings, runSteps } from "../support/steps";

/** A plain id means "click `li#id`"; anything with a `kind:` prefix is a step of its own (see support/steps.ts). */
const path = (...steps: string[]) => steps.map((s) => (/^(tab|input|type|mouse|drag|dropdown|range|js):/.test(s) ? s : `li#${s}`));

const SCREENS: string[][] = [
  [],
  ["Data"], ["Data", "tab:tab_number"], ["Data", "tab:tab_name"], ["Data", "tab:tab_mhh"], ["Data", "tab:tab_mhh", "abbset"],
  ["Data", "format"], ["Data", "format", "tab:tab_HPS"], ["Data", "order"], ["Data", "order", "tab:tab_HPS"],
  ["Design"], ["Design", "font"],
  ["Design", "color"], ["Design", "color", "tab:tab_table"], ["Design", "color", "tab:tab_graph"],
  ["Design", "opacity"], ["Design", "opacity", "tab:tab_table"], ["Design", "opacity", "tab:tab_graph"],
  ["Design", "size"], ["Design", "size", "tab:tab_table"], ["Design", "size", "tab:tab_graph"],
  ["Design", "cells"], ["Design", "cells", "tab:tab_width"], ["Design", "cells", "tab:tab_padding"], ["Design", "cells", "tab:tab_align"],
  ["Design", "shape"], ["Design", "shape", "tab:tab_table"], ["Design", "shape", "tab:tab_graph"],
  ["Design", "advanced"], ["Design", "advanced", "tab:tab_table"], ["Design", "advanced", "tab:tab_graph"],
  ["Design", "raid"], ["Design", "raid", "tab:tab_color"], ["Design", "raid", "tab:tab_opacity"], ["Design", "raid", "tab:tab_size"],
  ["Overlay"], ["Tool"], ["Tool", "custom"],
];

test.describe("settings pages", () => {
  for (const steps of SCREENS) {
    test(steps.join(" > ") || "settings (top)", async ({ app }) => {
      await app.matchScreenshot(`screens/${steps.join("-") || "top"}`, {
        after: async (page) => { await openSettings(page); await runSteps(page, path(...steps)); },
      });
    });
  }
});

// The colour picker. Coordinates are page pixels at the fixed 1100x640 window: the "Background" box on the
// colour page sits at (645, 319), so its panel opens below it, with the pad at (659, 365) and the slider at x = 913.
const COLOR_PAGE = ["Design", "color"];
const PICKER: string[][] = [
  [...COLOR_PAGE, "input:navBg"],
  [...COLOR_PAGE, "input:navBg", "mouse:759,425"],
  [...COLOR_PAGE, "input:navBg", "mouse:759,425", "mouse:921,400"],
  [...COLOR_PAGE, "input:accent"],
  [...COLOR_PAGE, "input:accent", "mouse:700,440"],
  [...COLOR_PAGE, "input:navBg", "drag:700,380>850,480"],
  [...COLOR_PAGE, "input:navBg", "drag:921,380>921,500"],
  [...COLOR_PAGE, "input:navBg", "type:navBg=ff8000"],
  [...COLOR_PAGE, "input:navBg", "type:navBg=FfA0b1"],
  [...COLOR_PAGE, "input:accent", "type:accent=123456"],
  ["Design", "raid", "tab:tab_color", "input:view24BgYOU", "type:view24BgYOU=00ff88"],
];
test.describe("colour picker", () => {
  for (const steps of PICKER) {
    test(steps.slice(2).join(" > "), async ({ app }) => {
      await app.matchScreenshot(`picker/${steps.slice(2).join("-")}`, {
        after: async (page) => { await openSettings(page); await runSteps(page, path(...steps)); },
      });
    });
  }
});
