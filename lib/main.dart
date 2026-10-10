import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:gosh_core/gosh_core.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'controller.dart';
import 'desktop.dart';
import 'diagnostics.dart';
import 'preferences.dart';
import 'ui/app_shell.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  try {
    await Diagnostics.initialize();
    Diagnostics.record('app', 'startup');
    LicenseRegistry.addLicense(() async* {
      yield LicenseEntryWithLineBreaks([
        'Gosh USB Creator',
      ], await rootBundle.loadString('LICENSE'));
      final notices =
          jsonDecode(await rootBundle.loadString('assets/rust_licenses.json'))
              as List;
      for (final entry in notices.cast<Map<String, dynamic>>()) {
        yield LicenseEntryWithLineBreaks(
          (entry['packages'] as List).cast<String>(),
          entry['text'] as String,
        );
      }
    });
    final preferences = Preferences(await SharedPreferences.getInstance());
    await prepareDesktop(preferences);
    final controller = AppController(RustBackend(), preferences);
    runApp(GoshApp(controller: controller));
    unawaited(controller.initialize());
  } catch (error, stack) {
    debugPrint('Gosh startup: $error\n$stack');
    runApp(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: Padding(
              padding: const EdgeInsets.all(32),
              child: SelectableText(
                'Gosh USB Creator could not start.\n$error',
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class GoshApp extends StatelessWidget {
  const GoshApp({super.key, required this.controller});
  final AppController controller;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: controller,
    builder: (context, _) => MaterialApp(
      title: 'Gosh USB Creator',
      debugShowCheckedModeBanner: false,
      themeMode: controller.preferences.theme,
      theme: _theme(Brightness.light),
      darkTheme: _theme(Brightness.dark),
      home: AppShell(controller: controller),
    ),
  );
  ThemeData _theme(Brightness brightness) {
    final colors = ColorScheme.fromSeed(
      seedColor: const Color(0xff24764a),
      brightness: brightness,
    );
    return ThemeData(
      colorScheme: colors,
      useMaterial3: true,
      brightness: brightness,
      visualDensity: VisualDensity.compact,
      inputDecorationTheme: const InputDecorationTheme(
        border: OutlineInputBorder(),
        isDense: true,
      ),
      cardTheme: CardThemeData(
        elevation: 0,
        margin: EdgeInsets.zero,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: colors.outlineVariant),
        ),
      ),
    );
  }
}
