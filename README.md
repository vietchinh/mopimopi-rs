# DEV NOTE

THIS INITIAL CODE IS FULL CONVERSION FROM MOPIMOPI TO DIOXUS RUST BY CLAUDE AI, SEE PULL REQUEST OF ME TRYING TO SIMPLIFY THE CODE.
USERS OF THIS BRANCH ARE BETA TESTING CODE THAT IS IN PROGRESS OF BEING VERIFIED BY THE DEV.

# Development

Your new bare-bones project includes minimal organization with a single `main.old` file and a few assets.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # main.rs is the entry point to your application and currently contains all components for the app
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

