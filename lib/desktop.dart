import 'dart:io';
import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';
import 'preferences.dart';

Future<void> prepareDesktop(Preferences preferences) async {
  await windowManager.ensureInitialized();
  final width = (preferences.storage.getDouble('windowWidth') ?? 940).clamp(
    720.0,
    2560.0,
  );
  final height = (preferences.storage.getDouble('windowHeight') ?? 780).clamp(
    640.0,
    1600.0,
  );
  await windowManager.waitUntilReadyToShow(
    WindowOptions(
      title: 'Gosh USB Creator',
      size: Size(width, height),
      minimumSize: const Size(720, 640),
      center: true,
      backgroundColor: Colors.transparent,
    ),
    () async {
      await windowManager.show();
      await windowManager.focus();
      if (preferences.storage.getBool('windowMaximized') ?? false) {
        await windowManager.maximize();
      }
    },
  );
  await windowManager.setPreventClose(true);
}

bool get usesCommandKey => Platform.isMacOS;
