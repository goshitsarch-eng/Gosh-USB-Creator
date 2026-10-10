import 'dart:async';
import 'package:flutter_test/flutter_test.dart';
import 'package:gosh_core/gosh_core.dart';
import 'package:gosh_usb_creator/controller.dart';
import 'package:gosh_usb_creator/preferences.dart';
import 'package:gosh_usb_creator/models.dart';
import 'package:shared_preferences/shared_preferences.dart';

class FakeBackend implements Backend {
  FakeBackend(this.handler);
  final Future<Map<String, dynamic>> Function(Map<String, dynamic>) handler;
  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) =>
      handler(request);
}

Map<String, dynamic> image(String name) => {
  'path': name,
  'name': name,
  'size': 512,
  'format': 'Unknown',
  'recognized': false,
  'warnings': <String>[],
};
void main() {
  test('checksum expectation and immutable image state', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = Preferences(await SharedPreferences.getInstance());
    final c = AppController(
      FakeBackend(
        (r) async =>
            r['operation'] == 'inspect' ? image('one') : {'checksum': 'abc'},
      ),
      prefs,
    );
    await c.selectImage('one');
    await c.calculate();
    expect(c.checksum, 'abc');
    c.setExpected(' ABC ');
    expect(c.checksumMatches, true);
    await c.selectImage('two');
    expect(c.checksum, isNull);
    c.dispose();
  });
  test(
    'busy input prevents racing source replacement and duplicate checksum',
    () async {
      SharedPreferences.setMockInitialValues({});
      final prefs = Preferences(await SharedPreferences.getInstance());
      final completed = Completer<Map<String, dynamic>>();
      var calls = 0;
      final c = AppController(
        FakeBackend((r) {
          calls++;
          return completed.future;
        }),
        prefs,
      );
      final selection = c.selectImage('one');
      await c.selectImage('two');
      expect(calls, 1);
      completed.complete(image('one'));
      await selection;
      expect(c.image!.name, 'one');
      c.dispose();
    },
  );
  test('legacy preferences import preserves backup and verifies schema', () async {
    SharedPreferences.setMockInitialValues({});
    final prefs = Preferences(await SharedPreferences.getInstance());
    await prefs.import(
      '{"gosh-usb-theme":"dark","gosh-usb-mode":"advanced","gosh-usb-verify-default":"false","gosh-usb-auto-eject":"true","gosh-usb-show-notification":"false"}',
    );
    expect(prefs.advanced, true);
    expect(prefs.verify, false);
    expect(prefs.autoEject, true);
    expect(prefs.storage.getString('settingsImportBackup'), isNotNull);
    await expectLater(
      prefs.import('{"schema":99}'),
      throwsA(isA<FormatException>()),
    );
    expect(prefs.advanced, true);
    await prefs.restoreBackup();
    expect(prefs.advanced, false);
    expect(prefs.verify, true);
    await expectLater(prefs.import('[]'), throwsA(isA<FormatException>()));
    await prefs.import(
      '{"gosh-usb-theme":null,"gosh-usb-mode":null,"gosh-usb-verify-default":null,"gosh-usb-auto-eject":null,"gosh-usb-show-notification":null}',
    );
    expect(prefs.theme.name, 'system');
    expect(prefs.verify, true);
    await expectLater(
      prefs.import('{"gosh-usb-verify-default":123}'),
      throwsA(isA<FormatException>()),
    );
  });
  test(
    'write startup locks inputs and queues cancellation until the job exists',
    () async {
      SharedPreferences.setMockInitialValues({});
      final start = Completer<Map<String, dynamic>>();
      final requests = <Map<String, dynamic>>[];
      final c = AppController(
        FakeBackend((r) async {
          requests.add(r);
          return switch (r['operation']) {
            'write' => start.future,
            'cancel' => <String, dynamic>{},
            'poll' => {'phase': 'cancelled'},
            _ => image('source'),
          };
        }),
        Preferences(await SharedPreferences.getInstance()),
      );
      await c.selectImage('source');
      c.selectDevice(
        DeviceInfo.fromJson({
          'path': '/dev/fake',
          'name': 'fixture',
          'size': 1024,
          'identity': 'one',
          'mount_points': <String>[],
        }),
      );
      expect(c.canWrite, true);
      final writing = c.write();
      expect(c.writing, true);
      expect(c.canWrite, false);
      await c.selectImage('replacement');
      await c.write();
      await c.cancel();
      expect(requests.where((r) => r['operation'] == 'write').length, 1);
      expect(requests.where((r) => r['operation'] == 'cancel'), isEmpty);
      start.complete({'id': 42});
      await writing;
      expect(requests.singleWhere((r) => r['operation'] == 'cancel')['id'], 42);
      expect(c.writing, false);
      expect(c.image!.path, 'source');
      c.dispose();
    },
  );
}
