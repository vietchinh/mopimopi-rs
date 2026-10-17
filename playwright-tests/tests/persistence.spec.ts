// What clicking a control stores. Each case performs the same clicks in both apps and compares which settings
// changed and to what: including the type (the original stores a chosen option as text, `"1"`, not a number).
import { test } from "../support/fixtures";
import { changedSettings, openSettings, readSettings, runSteps } from "../support/steps";
import { settingsFor } from "../support/settings";

const before = settingsFor({});
const CASES: [name: string, steps: string[]][] = [
  ["checkbox: combine pets", ["li#Data", "js:li#pets"]],
  ["checkbox: hide names", ["li#Data", "tab:tab_name", "js:li#hideName"]],
  ["checkbox: rank prefix", ["li#Data", "tab:tab_name", "js:li#rank"]],
  ["choice: damage type, 2nd option", ["li#Data", "tab:tab_number", "js:li#dpsType", "dropdown:2"]],
  ["choice: colour palette, 2nd option", ["li#Design", "li#color", "tab:tab_graph", "js:li#palette", "dropdown:2"]],
  ["column switched on", ["li#Data", "li#format", "js:li#DPS-mergedLast10DPS"]],
  ["column moved up", ["li#Data", "li#order", "ul li:nth-child(2) .UBtn"]],
  ["slider", ["li#Design", "li#size", "range:li#sizeNav input[type=range]=60"]],
  ["colour typed", ["li#Design", "li#color", "type:navBg=ff8000"]],
  ["switch: resizing arrow", ["li#Overlay", "js:li#arrow"]],
  ["switch: auto hide", ["li#Overlay", "js:li#autoHide"]],
];

for (const [name, steps] of CASES) {
  test(name, async ({ app }) => {
    await app.matchValue(`persistence/${name}`, {}, async (page) => {
      await openSettings(page);
      await runSteps(page, steps);
      return changedSettings(before, await readSettings(page));
    });
  });
}
