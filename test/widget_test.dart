import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gosh_core/gosh_core.dart';
import 'package:gosh_usb_creator/controller.dart';
import 'package:gosh_usb_creator/preferences.dart';
import 'package:gosh_usb_creator/models.dart' as domain;
import 'package:gosh_usb_creator/ui/app_shell.dart';
import 'package:shared_preferences/shared_preferences.dart';

class EmptyBackend implements Backend {
  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async => {
    'devices': <Map<String, dynamic>>[],
  };
}

void main() {
  testWidgets(
    'empty target disables write, navigation and advanced settings work',
    (tester) async {
      tester.view.physicalSize = const Size(1100, 1000);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      SharedPreferences.setMockInitialValues({});
      final c = AppController(
        EmptyBackend(),
        Preferences(await SharedPreferences.getInstance()),
      );
      await tester.pumpWidget(
        ListenableBuilder(
          listenable: c,
          builder: (context, _) =>
              MaterialApp(home: AppShell(controller: c, manageWindow: false)),
        ),
      );
      expect(find.text('Write an image to USB'), findsOneWidget);
      final write = tester.widget<FilledButton>(
        find.widgetWithText(FilledButton, 'Write image'),
      );
      expect(write.onPressed, isNull);
      c.image = domain.ImageInfo.fromJson({
        'path': 'fixture.img',
        'name': 'fixture.img',
        'size': 2048,
        'format': 'Unknown',
        'recognized': false,
        'warnings': ['Unknown image signature.'],
      });
      c.selected = domain.DeviceInfo.fromJson({
        'path': '/dev/fake',
        'name': 'fixture',
        'size': 1024,
        'identity': 'one',
        'mount_points': <String>[],
      });
      c.changed();
      await tester.pump();
      expect(
        find.text('Image exceeds the selected drive’s capacity.'),
        findsOneWidget,
      );
      expect(find.text('Unknown image signature.'), findsOneWidget);
      await tester.tap(find.widgetWithText(ListTile, 'Settings'));
      await tester.pumpAndSettle();
      expect(
        find.text('Follow the system theme, or choose a fixed appearance.'),
        findsOneWidget,
      );
      await tester.tap(find.widgetWithText(SwitchListTile, 'Advanced mode'));
      await tester.pumpAndSettle();
      expect(find.text('Auto-eject drive'), findsOneWidget);
      c.report('Readable failure');
      await tester.pump();
      expect(find.text('Readable failure'), findsOneWidget);
      await tester.pumpWidget(const SizedBox());
      c.dispose();
    },
  );
}
