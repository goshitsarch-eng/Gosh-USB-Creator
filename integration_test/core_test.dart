import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:gosh_core/gosh_core.dart';
import 'package:integration_test/integration_test.dart';
import 'package:gosh_usb_creator/controller.dart';
import 'package:gosh_usb_creator/preferences.dart';
import 'package:gosh_usb_creator/ui/app_shell.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  test(
    'real native bridge handles Unicode files, checksums, and typed errors',
    () async {
      final dir = await Directory.systemTemp.createTemp('gosh bridge ü ');
      final file = File('${dir.path}/image ü.img');
      final bytes = List<int>.filled(4096, 0);
      bytes[510] = 0x55;
      bytes[511] = 0xaa;
      await file.writeAsBytes(bytes);
      final backend = RustBackend();
      try {
        final info = await backend.call({
          'operation': 'inspect',
          'path': file.path,
        });
        expect(info['format'], 'Disk Image (MBR)');
        expect(info['size'], 4096);
        final hash = await backend.call({
          'operation': 'checksum',
          'path': file.path,
          'algorithm': 'sha256',
        });
        expect((hash['checksum'] as String).length, 64);
        await expectLater(
          backend.call({'operation': 'inspect', 'path': '${dir.path}/missing'}),
          throwsA(isA<CoreFailure>()),
        );
        await expectLater(
          backend.call({'operation': 'unknown'}),
          throwsA(isA<CoreFailure>()),
        );
        final write = await backend.call({
          'operation': 'write',
          'request': {
            'source': file.path,
            'device': {
              'path': file.path,
              'name': 'not a device',
              'size': 4096,
              'identity': 'fake',
              'mount_points': <String>[],
            },
            'verify': true,
            'auto_eject': false,
          },
        });
        Map<String, dynamic> job;
        final deadline = DateTime.now().add(const Duration(seconds: 5));
        do {
          await Future<void>.delayed(const Duration(milliseconds: 20));
          job = await backend.call({'operation': 'poll', 'id': write['id']});
        } while (job['phase'] == 'preparing' &&
            DateTime.now().isBefore(deadline));
        expect(job['phase'], 'error');
        expect((job['error'] as Map)['kind'], 'device_changed');
        expect(await file.readAsBytes(), bytes);
        final results = await Future.wait(
          List.generate(
            8,
            (_) => backend.call({
              'operation': 'checksum',
              'path': file.path,
              'algorithm': 'md5',
            }),
          ),
        );
        expect(results.map((r) => r['checksum']).toSet().length, 1);
      } finally {
        await dir.delete(recursive: true);
      }
    },
  );
  testWidgets(
    'desktop image controls use the real core for inspection and hashing',
    (tester) async {
      final dir = await Directory.systemTemp.createTemp('gosh ui ü ');
      final file = File('${dir.path}/fixture ü.img');
      final bytes = List<int>.filled(8192, 0);
      bytes[510] = 0x55;
      bytes[511] = 0xaa;
      await file.writeAsBytes(bytes);
      SharedPreferences.setMockInitialValues({'advanced': true});
      final c = AppController(
        RustBackend(),
        Preferences(await SharedPreferences.getInstance()),
      );
      try {
        await c.selectImage(file.path);
        await c.refresh();
        await tester.pumpWidget(
          ListenableBuilder(
            listenable: c,
            builder: (context, _) =>
                MaterialApp(home: AppShell(controller: c, manageWindow: false)),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.text('fixture ü.img'), findsOneWidget);
        expect(find.text('Disk Image (MBR)'), findsOneWidget);
        await tester.tap(find.text('Disk Image (MBR)'));
        await tester.pumpAndSettle();
        expect(find.byType(AlertDialog), findsOneWidget);
        await tester.tap(find.text('Close'));
        await tester.pumpAndSettle();
        await tester.ensureVisible(find.text('Calculate'));
        await tester.tap(find.text('Calculate'));
        await tester.pumpAndSettle();
        final deadline = DateTime.now().add(const Duration(seconds: 5));
        while (c.hashing && DateTime.now().isBefore(deadline)) {
          await tester.pump(const Duration(milliseconds: 50));
        }
        expect(c.hashing, false);
        expect(c.checksum?.length, 64);
        expect(find.text(c.checksum!), findsOneWidget);
        await tester.enterText(
          find.byType(TextFormField),
          c.checksum!.toUpperCase(),
        );
        await tester.pump();
        expect(find.text('Checksums match'), findsOneWidget);
        await tester.drag(find.byType(ListView).first, const Offset(0, 1000));
        await tester.pumpAndSettle();
        await tester.tap(find.byTooltip('Clear source image'));
        await tester.pumpAndSettle();
        expect(c.image, isNull);
      } finally {
        await tester.pumpWidget(const SizedBox());
        c.dispose();
        await dir.delete(recursive: true);
      }
    },
  );
}
