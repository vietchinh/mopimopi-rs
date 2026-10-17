# Fonts

| File | What | Licence | Source |
|---|---|---|---|
| `DS-DIGIB.ttf`, `DS-DIGIB.woff` | the clock's digits | as in the original overlay | the original MopiMopi |
| `RobotoCondensed-latin-400.woff2`, `RobotoCondensed-latin-ext-400.woff2` | the default text font (Roboto Condensed, regular), the `latin` and `latin-ext` subsets Google Fonts serves | SIL Open Font License 1.1 (`RobotoCondensed-OFL.txt`) | `@fontsource/roboto-condensed` 5.3.0 |
| `MaterialIcons-Regular.woff2` | the icon font | Apache License 2.0 | Google's Material Icons |

They are bundled so the overlay needs no network for its fonts (it used to load both from fonts.googleapis.com). `@font-face` rules are written in
`src/presentation/ui/app_shell.rs`, because a rule inside a hashed stylesheet could not name them with a correct URL.
