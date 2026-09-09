# Kron

A desktop time-tracking application for **macOS** and **Windows**, built with Tauri 2, React, and TypeScript.

Track time against clients, round sessions up to the next 15 minutes, and review day / week / month reports. The timer also lives in the macOS menu bar or the Windows notification area.

## Prerequisites

### Both platforms

- [Node.js](https://nodejs.org/) 18 or newer
- [Rust](https://rustup.rs) (stable), via rustup

After installing rustup, confirm `cargo` is on your `PATH`:

```bash
cargo --version
```

If `npm run tauri dev` fails with `failed to run command cargo metadata`, the terminal was opened before Rust was installed. Run `source ~/.cargo/env` (macOS/Linux) or open a new terminal, then try again.

### macOS

- Xcode Command Line Tools: `xcode-select --install`

### Windows

- Windows 10 or 11 (64-bit)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload (MSVC)
- Rust target `stable-x86_64-pc-windows-msvc` (this is rustup’s default on 64-bit Windows)
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) — already installed on most Windows 10/11 machines. The NSIS installer can also download it if it is missing.
- Optional: [NSIS](https://nsis.sourceforge.io/) is pulled in by the Tauri CLI when you run a production build; you do not need to install it by hand for `tauri dev`.

Build Kron **on Windows**. Cross-compiling the Windows app from macOS is not supported here.

## Development

From the repository root:

```bash
npm install
npm run tauri dev
```

The first compile takes several minutes. The window opens when the backend is ready. Closing the window hides it; use **Open Kron** from the tray (or left-click the tray icon on Windows) to bring it back, and **Quit** to exit.

## Production build

```bash
npm run tauri build
```

Tauri only produces installers for the OS you are building on.

| Platform | Artifacts |
|---|---|
| macOS | `src-tauri/target/release/bundle/macos/Kron.app` and `src-tauri/target/release/bundle/dmg/` |
| Windows | `src-tauri/target/release/kron.exe` and `src-tauri/target/release/bundle/nsis/` (setup `.exe`) |

Unsigned Windows installers may show a SmartScreen warning. Signing requires a code-signing certificate.

## Data location

The SQLite database is stored in the OS application-data directory, not inside the app bundle:

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/com.kron.desktop/kron.db` |
| Windows | `%APPDATA%\com.kron.desktop\kron.db` |

## Tray behavior

- **macOS:** the menu extra shows a template clock icon plus a title (`Kron`, or `Client  HH:MM` while a timer runs). Click it to open the menu.
- **Windows:** the notification-area icon is the color app glyph. Hover for the same status as a tooltip (Windows does not draw title text next to tray icons). Left-click restores the window; right-click opens the menu.

## License

Kron is released under the [MIT License](LICENSE).
