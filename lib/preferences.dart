import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';

class Preferences {
  Preferences(this.storage);
  final SharedPreferences storage;
  ThemeMode get theme => switch (storage.getString('theme')) {
    'light' => ThemeMode.light,
    'dark' => ThemeMode.dark,
    _ => ThemeMode.system,
  };
  bool get advanced => storage.getBool('advanced') ?? false;
  bool get verify => storage.getBool('verify') ?? true;
  bool get autoEject => storage.getBool('autoEject') ?? false;
  bool get notifications => storage.getBool('notifications') ?? true;
  bool get hasBackup => storage.containsKey('settingsImportBackup');
  Future<void> setTheme(ThemeMode value) async {
    if (!await storage.setString('theme', value.name)) {
      throw StateError('Could not save theme.');
    }
  }

  Future<void> setBool(String key, bool value) async {
    if (!await storage.setBool(key, value)) {
      throw StateError('Could not save preference.');
    }
  }

  Map<String, dynamic> export() => {
    'schema': 1,
    'theme': theme.name,
    'advanced': advanced,
    'verify': verify,
    'autoEject': autoEject,
    'notifications': notifications,
  };
  Future<void> import(String source) async {
    final decoded = jsonDecode(source);
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('Settings must contain a JSON object.');
    }
    final json = decoded;
    const legacyBooleans = [
      'gosh-usb-verify-default',
      'gosh-usb-auto-eject',
      'gosh-usb-show-notification',
    ];
    const legacyKeys = ['gosh-usb-theme', 'gosh-usb-mode', ...legacyBooleans];
    final legacy = json.keys.any(legacyKeys.contains);
    if (legacy &&
        (legacyBooleans.any(
              (key) =>
                  json[key] != null && !['true', 'false'].contains(json[key]),
            ) ||
            (json['gosh-usb-mode'] != null &&
                !['standard', 'advanced'].contains(json['gosh-usb-mode'])))) {
      throw const FormatException('Legacy settings contain an invalid value.');
    }
    final data = legacy
        ? {
            'theme': json['gosh-usb-theme'] ?? 'system',
            'advanced': json['gosh-usb-mode'] == 'advanced',
            'verify': json['gosh-usb-verify-default'] != 'false',
            'autoEject': json['gosh-usb-auto-eject'] == 'true',
            'notifications': json['gosh-usb-show-notification'] != 'false',
          }
        : json;
    if ((!legacy && data['schema'] != 1) ||
        !['light', 'dark', 'system'].contains(data['theme']) ||
        [
          'advanced',
          'verify',
          'autoEject',
          'notifications',
        ].any((k) => data[k] is! bool)) {
      throw const FormatException(
        'Choose a Gosh settings export or legacy localStorage export.',
      );
    }
    // Keep a recoverable prior preferences snapshot before applying an import.
    final prior = export();
    if (!await storage.setString('settingsImportBackup', jsonEncode(prior))) {
      throw StateError('Could not save the settings backup.');
    }
    try {
      await _apply(data);
    } catch (_) {
      await _apply(prior);
      rethrow;
    }
  }

  Future<void> _apply(Map<String, dynamic> data) async {
    await setTheme(ThemeMode.values.firstWhere((t) => t.name == data['theme']));
    for (final key in ['advanced', 'verify', 'autoEject', 'notifications']) {
      await setBool(key, data[key] as bool);
    }
  }

  Future<void> restoreBackup() async {
    final backup = storage.getString('settingsImportBackup');
    if (backup == null) throw StateError('No settings backup is available.');
    await import(backup);
  }
}
