import { confirm } from "@tauri-apps/plugin-dialog";

// wry's WKUIDelegate does not implement the JavaScript confirm panel, so window.confirm
// never shows a dialog and always resolves to false. Confirmations must go through the
// dialog plugin instead.
export async function confirmAction(message: string, okLabel: string): Promise<boolean> {
  return confirm(message, { title: "Kron", kind: "warning", okLabel });
}
