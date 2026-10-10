import Cocoa
import FlutterMacOS

@main
class AppDelegate: FlutterAppDelegate {
  override func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
    // Route the native application menu / Command-Q through window_manager's
    // close event, so Dart can cancel and await an active write before closing.
    if let window = sender.windows.first(where: { $0.isVisible && $0.contentViewController is FlutterViewController }) {
      window.performClose(nil)
      return .terminateCancel
    }
    return .terminateNow
  }

  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return true
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }
}
