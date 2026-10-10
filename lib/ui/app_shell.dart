import 'dart:async';
import 'dart:convert';
import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:local_notifier/local_notifier.dart';
import 'package:window_manager/window_manager.dart';
import '../controller.dart';
import '../desktop.dart';
import 'settings_screen.dart';
import 'write_screen.dart';

class _OpenIntent extends Intent {
  const _OpenIntent();
}

class _RefreshIntent extends Intent {
  const _RefreshIntent();
}

class _WriteIntent extends Intent {
  const _WriteIntent();
}

class _SettingsIntent extends Intent {
  const _SettingsIntent();
}

class _QuitIntent extends Intent {
  const _QuitIntent();
}

class AppShell extends StatefulWidget {
  const AppShell({
    super.key,
    required this.controller,
    this.manageWindow = true,
  });
  final AppController controller;
  final bool manageWindow;
  @override
  State<AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<AppShell> with WindowListener {
  bool settings = false, _closing = false, _dialogOpen = false;
  Timer? _windowSave;
  int? _notifiedJob;
  AppController get c => widget.controller;
  @override
  void initState() {
    super.initState();
    if (widget.manageWindow) windowManager.addListener(this);
    c.addListener(_onState);
  }

  @override
  void dispose() {
    c.removeListener(_onState);
    _windowSave?.cancel();
    if (widget.manageWindow) windowManager.removeListener(this);
    super.dispose();
  }

  void _onState() {
    if (c.progress?['phase'] == 'complete' && c.jobId != _notifiedJob) {
      _notifiedJob = c.jobId;
      if (c.preferences.advanced && c.preferences.notifications) {
        unawaited(_notify());
      }
    }
  }

  Future<void> _notify() async {
    try {
      await localNotifier.setup(
        appName: 'Gosh USB Creator',
        shortcutPolicy: ShortcutPolicy.requireCreate,
      );
      await LocalNotification(
        title: 'Image written',
        body:
            'Gosh USB Creator finished writing the selected image. Check the app for eject status.',
      ).show();
    } catch (e) {
      c.report('Notification could not be shown: $e');
    }
  }

  Future<void> _open() async {
    if (c.inputBusy || _dialogOpen) return;
    _dialogOpen = true;
    try {
      final file = await openFile(
        acceptedTypeGroups: const [
          XTypeGroup(label: 'Disk images', extensions: ['iso', 'img']),
          XTypeGroup(label: 'All files'),
        ],
      );
      if (file != null) await c.selectImage(file.path);
    } catch (e) {
      c.report(e);
    } finally {
      _dialogOpen = false;
    }
  }

  Future<void> _write() async {
    if (!c.canWrite || _dialogOpen) return;
    _dialogOpen = true;
    final image = c.image!;
    final device = c.selected!;
    final confirmed = await showDialog<bool>(
      context: context,
      barrierDismissible: false,
      builder: (context) => AlertDialog(
        title: const Text('Erase and write this drive?'),
        content: Text(
          'All data on ${device.name} (${device.path}, ${device.size} bytes) will be erased.\n\nSource: ${image.name}\n\nCheck the target carefully before continuing.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Erase and write'),
          ),
        ],
      ),
    );
    _dialogOpen = false;
    if (confirmed == true &&
        c.image == image &&
        c.selected?.path == device.path &&
        c.selected?.identity == device.identity &&
        c.selected?.size == device.size) {
      await c.write();
    }
  }

  Future<void> _import() async {
    try {
      final f = await openFile(
        acceptedTypeGroups: const [
          XTypeGroup(label: 'Settings JSON', extensions: ['json']),
        ],
      );
      if (f == null) return;
      if (await f.length() > 65536) {
        throw const FormatException('Settings file is too large.');
      }
      await c.preferences.import(await f.readAsString());
      c.changed();
    } catch (e) {
      c.report(e);
    }
  }

  Future<void> _export() async {
    try {
      final location = await getSaveLocation(
        suggestedName: 'gosh-usb-settings.json',
        acceptedTypeGroups: const [
          XTypeGroup(label: 'Settings JSON', extensions: ['json']),
        ],
      );
      if (location == null) return;
      await XFile.fromData(
        utf8.encode(
          const JsonEncoder.withIndent('  ').convert(c.preferences.export()),
        ),
        mimeType: 'application/json',
      ).saveTo(location.path);
    } catch (e) {
      c.report(e);
    }
  }

  @override
  void onWindowResize() {
    _windowSave?.cancel();
    _windowSave = Timer(
      const Duration(milliseconds: 400),
      () => unawaited(_saveWindow()),
    );
  }

  @override
  void onWindowMaximize() => unawaited(_saveWindow());
  @override
  void onWindowUnmaximize() => unawaited(_saveWindow());
  Future<void> _saveWindow() async {
    try {
      final maximized = await windowManager.isMaximized();
      await c.preferences.storage.setBool('windowMaximized', maximized);
      if (!maximized) {
        final size = await windowManager.getSize();
        await c.preferences.storage.setDouble('windowWidth', size.width);
        await c.preferences.storage.setDouble('windowHeight', size.height);
      }
    } catch (e) {
      c.report('Window state could not be saved: $e');
    }
  }

  @override
  void onWindowClose() => unawaited(_quit());
  Future<void> _quit() async {
    if (_closing || _dialogOpen) return;
    _closing = true;
    if (c.writing) {
      final cancel = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('A write is still running'),
          content: const Text(
            'Cancelling can leave an incomplete USB image. Wait for the operation to stop before closing.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Keep writing'),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: const Text('Cancel write and quit'),
            ),
          ],
        ),
      );
      if (cancel != true) {
        _closing = false;
        return;
      }
      await c.cancel();
      while (c.writing && mounted) {
        await Future<void>.delayed(const Duration(milliseconds: 200));
        await c.poll();
      }
    }
    if (widget.manageWindow) {
      await _saveWindow();
      await windowManager.destroy();
    }
    _closing = false;
  }

  @override
  Widget build(BuildContext context) {
    final mod = usesCommandKey;
    return Shortcuts(
      shortcuts: {
        SingleActivator(LogicalKeyboardKey.keyO, control: !mod, meta: mod):
            const _OpenIntent(),
        SingleActivator(LogicalKeyboardKey.keyR, control: !mod, meta: mod):
            const _RefreshIntent(),
        SingleActivator(LogicalKeyboardKey.enter, control: !mod, meta: mod):
            const _WriteIntent(),
        SingleActivator(LogicalKeyboardKey.comma, control: !mod, meta: mod):
            const _SettingsIntent(),
        SingleActivator(LogicalKeyboardKey.keyQ, control: !mod, meta: mod):
            const _QuitIntent(),
      },
      child: Actions(
        actions: {
          _QuitIntent: CallbackAction<_QuitIntent>(onInvoke: (_) => _quit()),
          _OpenIntent: CallbackAction<_OpenIntent>(
            onInvoke: (_) {
              unawaited(_open());
              return null;
            },
          ),
          _RefreshIntent: CallbackAction<_RefreshIntent>(
            onInvoke: (_) {
              unawaited(c.refresh());
              return null;
            },
          ),
          _WriteIntent: CallbackAction<_WriteIntent>(
            onInvoke: (_) {
              unawaited(_write());
              return null;
            },
          ),
          _SettingsIntent: CallbackAction<_SettingsIntent>(
            onInvoke: (_) {
              setState(() => settings = true);
              return null;
            },
          ),
        },
        child: Focus(
          autofocus: true,
          child: Scaffold(
            body: SafeArea(
              child: Column(
                children: [
                  Align(
                    alignment: Alignment.centerLeft,
                    child: MenuBar(
                      style: const MenuStyle(
                        elevation: WidgetStatePropertyAll(0),
                      ),
                      children: [
                        SubmenuButton(
                          menuChildren: [
                            MenuItemButton(
                              onPressed: c.inputBusy ? null : _open,
                              child: const Text('Open image…'),
                            ),
                            MenuItemButton(
                              onPressed: c.writing ? null : c.refresh,
                              child: const Text('Refresh drives'),
                            ),
                            MenuItemButton(
                              onPressed: _quit,
                              child: const Text('Quit'),
                            ),
                          ],
                          child: const Text('File'),
                        ),
                        SubmenuButton(
                          menuChildren: [
                            MenuItemButton(
                              onPressed: () => setState(() => settings = false),
                              child: const Text('Write image'),
                            ),
                            MenuItemButton(
                              onPressed: () => setState(() => settings = true),
                              child: const Text('Settings'),
                            ),
                          ],
                          child: const Text('View'),
                        ),
                        SubmenuButton(
                          menuChildren: [
                            MenuItemButton(
                              onPressed: () => showAboutDialog(
                                context: context,
                                applicationName: 'Gosh USB Creator',
                                applicationVersion: '3.0.0',
                                applicationLegalese: 'AGPL-3.0',
                              ),
                              child: const Text('About Gosh USB Creator'),
                            ),
                          ],
                          child: const Text('Help'),
                        ),
                      ],
                    ),
                  ),
                  if (c.error != null)
                    Material(
                      color: Theme.of(context).colorScheme.errorContainer,
                      child: Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 16,
                          vertical: 8,
                        ),
                        child: Row(
                          children: [
                            const Icon(Icons.error_outline),
                            const SizedBox(width: 12),
                            Expanded(child: SelectableText(c.error!)),
                            IconButton(
                              tooltip: 'Dismiss error',
                              onPressed: () {
                                c.error = null;
                                c.changed();
                              },
                              icon: const Icon(Icons.close),
                            ),
                          ],
                        ),
                      ),
                    ),
                  Expanded(
                    child: Row(
                      children: [
                        SizedBox(
                          width: 190,
                          child: Material(
                            color: Theme.of(
                              context,
                            ).colorScheme.surfaceContainerLow,
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.stretch,
                              children: [
                                const Padding(
                                  padding: EdgeInsets.fromLTRB(20, 28, 20, 24),
                                  child: Column(
                                    crossAxisAlignment:
                                        CrossAxisAlignment.start,
                                    children: [
                                      Icon(Icons.usb, size: 30),
                                      SizedBox(height: 12),
                                      Text(
                                        'Gosh USB Creator',
                                        style: TextStyle(
                                          fontWeight: FontWeight.w600,
                                          fontSize: 16,
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                                _nav(
                                  'Write image',
                                  Icons.drive_file_move_outline,
                                  !settings,
                                  () => setState(() => settings = false),
                                ),
                                _nav(
                                  'Settings',
                                  Icons.settings_outlined,
                                  settings,
                                  () => setState(() => settings = true),
                                ),
                                const Spacer(),
                                Padding(
                                  padding: const EdgeInsets.all(16),
                                  child: Column(
                                    crossAxisAlignment:
                                        CrossAxisAlignment.start,
                                    children: [
                                      const Text(
                                        'Mode',
                                        style: TextStyle(fontSize: 12),
                                      ),
                                      const SizedBox(height: 8),
                                      DropdownButtonFormField<bool>(
                                        initialValue: c.preferences.advanced,
                                        key: ValueKey(c.preferences.advanced),
                                        items: const [
                                          DropdownMenuItem(
                                            value: false,
                                            child: Text('Standard'),
                                          ),
                                          DropdownMenuItem(
                                            value: true,
                                            child: Text('Advanced'),
                                          ),
                                        ],
                                        onChanged: c.writing
                                            ? null
                                            : (v) async {
                                                try {
                                                  await c.preferences.setBool(
                                                    'advanced',
                                                    v!,
                                                  );
                                                  c.changed();
                                                } catch (e) {
                                                  c.report(e);
                                                }
                                              },
                                      ),
                                    ],
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                        const VerticalDivider(width: 1),
                        Expanded(
                          child: settings
                              ? SettingsScreen(
                                  controller: c,
                                  importSettings: _import,
                                  exportSettings: _export,
                                )
                              : WriteScreen(
                                  controller: c,
                                  open: _open,
                                  write: _write,
                                ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _nav(String text, IconData icon, bool selected, VoidCallback action) =>
      Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
        child: ListTile(
          dense: true,
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
          selected: selected,
          selectedTileColor: Theme.of(context).colorScheme.secondaryContainer,
          leading: Icon(icon),
          title: Text(text),
          onTap: action,
        ),
      );
}
