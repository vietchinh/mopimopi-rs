## Using the overlay

The overlay gets its data from **OverlayPlugin's WebSocket server**, and only from there. The address of that server is part of
the page's URL.

### 1. Start OverlayPlugin's WebSocket server

1. In ACT, install **OverlayPlugin** (Plugins, Plugin Listing, Get Plugins..., select Overlay Plugin, Download and Enable),
   then restart ACT.
2. Open **Plugins, OverlayPlugin WSServer**. Use IP `127.0.0.1` and port `10501`. If the overlay page is served over https
   (GitHub Pages), tick **Enable SSL** and click **Generate SSL Certificate**. Click **Start** and check that the status is
   **Running**.

### 2. Put the address in the URL

Add `?OVERLAY_WS=` and the server's address to the page's URL:

```
https://vietchinh.github.io/mopimopi-rs/?OVERLAY_WS=ws://127.0.0.1:10501/ws
https://vietchinh.github.io/mopimopi-rs/?OVERLAY_WS=wss://127.0.0.1:10501/ws      (Enable SSL ticked)
```

- The WSServer tab's URL generator (Stream/Local overlay) adds `OVERLAY_WS=` to an overlay URL for you, using `wss://` when
  Enable SSL is ticked. (From OverlayPlugin's source code; I have not used the generator itself.)
- `/ws` is added when the address has no path, so `?OVERLAY_WS=ws://127.0.0.1:10501` works too.
- Older links with `?HOST_PORT=ws://127.0.0.1:10501` still work and mean the same thing.
- The address is not typed into the page any more. Without it the start screen explains what to add and offers sample data.

### 3. Add it as an overlay (in the game) or open it in a browser

- **In the game:** Plugins, **OverlayPlugin.dll**, **New**. Enter a name, choose the preset **Custom** and the type
  **MiniParse** (in OverlayPlugin, that is simply the type for overlays that are a web page), and set the **URL** to the link
  from step 2. Move and resize the overlay, then tick **Enable clickthrough** and **Lock overlay**.
- **In a normal browser, OBS or on a phone:** just open the link from step 2.

The overlay connects by itself, reconnects by itself (after 1, 1, 2, 4, 8, then every 15 seconds) and tells you what is
happening on the start screen and, once data is on screen, in the top bar.

### ACT settings the overlay depends on

- **`YOU`:** in ACT, Options, Miscellaneous, enter `YOU` in capital letters and click Apply. The overlay identifies your own
  row by that name; without it the tables stay empty and the overlay says so.
- **Pets and shields:** in the FFXIV plugin settings the original guide asks for **Disable Combine Pets with Owner**
  ticked and **Disable Damage Shield estimates** unticked, so pet and shield graphs are accurate. A Parse Filter of
  **Alliance** is recommended for raid mode. (I have not checked whether these option names still exist in the current
  plugin; OverlayPlugin's own guide says no extra network-capture configuration is needed since patch 7.2.)
- **Ending encounters:** OverlayPlugin's guide recommends disabling ACT's encounter split timers (Options, Main
  Table/Encounters, General: untick both "Number of seconds to wait") and enabling **End ACT encounter after wipe** and
  **End ACT encounter out of combat** under OverlayPlugin.dll, Event Settings. Encounters then end reliably, which is what
  fills the History screen.

### The Capture and End-encounter buttons

- **End encounter** asks ACT to end the current encounter. OverlayPlugin only implements this on its legacy endpoint, so the
  page briefly connects to `/MiniParse` on the same server (same address and port as `OVERLAY_WS`) and sends the request.
  A toast says whether the request was sent. (Found in OverlayPlugin's source, `WSServer.cs`.) Ending encounters
  automatically, as described above, is usually better.
- **Capture** saves a PNG of the overlay: the page draws itself and downloads the file through the browser. The buttons are
  left out of the picture, as in the original. OverlayPlugin itself cannot take the screenshot for a WebSocket client
  (its source only logs that Capture is not supported). Fonts from other sites are replaced by the next font in the stack.
  Whether the download works inside OverlayPlugin's own overlay window has not been tested; a normal browser works.
- Both buttons appear when pinned in the settings (Advanced, buttons) or while the mouse is over the ⋮ button.

### What is not supported

Only OverlayPlugin's WebSocket is supported. The legacy ACTWebSocket / MiniParse data protocol and OverlayPlugin's in-game
`OverlayPluginApi` are gone: an overlay must have the address in its URL as described above.

Sources: the OverlayPlugin documentation (setup guide, FAQ, source) and the original MopiMopi guide slides.
