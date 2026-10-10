import 'package:flutter/material.dart';
import '../controller.dart';
import 'write_screen.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({
    super.key,
    required this.controller,
    required this.importSettings,
    required this.exportSettings,
  });
  final AppController controller;
  final VoidCallback importSettings, exportSettings;
  @override
  Widget build(BuildContext context) {
    final c = controller;
    final prefs = c.preferences;
    Future<void> toggle(String key, bool value) async {
      try {
        await prefs.setBool(key, value);
        c.changed();
      } catch (e) {
        c.report(e);
      }
    }

    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        Text('Settings', style: Theme.of(context).textTheme.headlineSmall),
        const SizedBox(height: 24),
        Section(
          'Appearance',
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Follow the system theme, or choose a fixed appearance.',
              ),
              const SizedBox(height: 16),
              SegmentedButton<ThemeMode>(
                segments: const [
                  ButtonSegment(
                    value: ThemeMode.light,
                    label: Text('Light'),
                    icon: Icon(Icons.light_mode_outlined),
                  ),
                  ButtonSegment(
                    value: ThemeMode.dark,
                    label: Text('Dark'),
                    icon: Icon(Icons.dark_mode_outlined),
                  ),
                  ButtonSegment(
                    value: ThemeMode.system,
                    label: Text('System'),
                    icon: Icon(Icons.settings_brightness_outlined),
                  ),
                ],
                selected: {prefs.theme},
                onSelectionChanged: (v) async {
                  try {
                    await prefs.setTheme(v.first);
                    c.changed();
                  } catch (e) {
                    c.report(e);
                  }
                },
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        Section(
          'Defaults',
          Column(
            children: [
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Advanced mode'),
                subtitle: const Text(
                  'Image signature details and post-write actions.',
                ),
                value: prefs.advanced,
                onChanged: c.writing ? null : (v) => toggle('advanced', v),
              ),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Verify after writing'),
                subtitle: const Text(
                  'Read back the written bytes and compare with the source.',
                ),
                value: prefs.verify,
                onChanged: c.writing ? null : (v) => toggle('verify', v),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        if (prefs.advanced) ...[
          Section(
            'Post-write actions',
            Column(
              children: [
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  title: const Text('Auto-eject drive'),
                  value: prefs.autoEject,
                  onChanged: c.writing ? null : (v) => toggle('autoEject', v),
                ),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  title: const Text('Notify on completion'),
                  value: prefs.notifications,
                  onChanged: c.writing
                      ? null
                      : (v) => toggle('notifications', v),
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
        ],
        Section(
          'Settings compatibility',
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Import a Gosh settings JSON file. A backup of your current preferences is saved before import; previous app data is preserved.',
              ),
              const SizedBox(height: 16),
              Wrap(
                spacing: 12,
                runSpacing: 12,
                children: [
                  OutlinedButton.icon(
                    onPressed: c.inputBusy ? null : importSettings,
                    icon: const Icon(Icons.file_open_outlined),
                    label: const Text('Import settings'),
                  ),
                  OutlinedButton.icon(
                    onPressed: exportSettings,
                    icon: const Icon(Icons.save_outlined),
                    label: const Text('Export settings'),
                  ),
                  OutlinedButton.icon(
                    onPressed: c.inputBusy || !prefs.hasBackup
                        ? null
                        : () async {
                            try {
                              await prefs.restoreBackup();
                              c.changed();
                            } catch (e) {
                              c.report(e);
                            }
                          },
                    icon: const Icon(Icons.restore),
                    label: const Text('Restore previous settings'),
                  ),
                ],
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        Section(
          'About',
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'Gosh USB Creator',
                style: TextStyle(fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 8),
              const Text('Version 3.0.0 · AGPL-3.0'),
              const SizedBox(height: 8),
              const Text(
                'Write and verify images on removable drives. No accounts, telemetry, or cloud services.',
              ),
              TextButton(
                onPressed: () => showLicensePage(
                  context: context,
                  applicationName: 'Gosh USB Creator',
                  applicationVersion: '3.0.0',
                ),
                child: const Text('Open-source licenses'),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
