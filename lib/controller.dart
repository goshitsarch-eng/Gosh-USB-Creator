import 'dart:async';
import 'diagnostics.dart';
import 'package:flutter/foundation.dart';
import 'package:gosh_core/gosh_core.dart';
import 'models.dart';
import 'preferences.dart';

class AppController extends ChangeNotifier {
  AppController(this.backend, this.preferences);
  final Backend backend;
  final Preferences preferences;
  ImageInfo? image;
  DeviceInfo? selected;
  List<DeviceInfo> devices = [];
  String algorithm = 'sha256';
  String expected = '';
  String? checksum, error;
  bool inspecting = false, hashing = false, scanning = false;
  int? jobId;
  Map<String, dynamic>? progress;
  Timer? _refresh, _poll;
  bool _disposed = false, _polling = false;
  bool _startingWrite = false, _cancelRequested = false;
  int _selectionVersion = 0, _scanVersion = 0;
  bool get writing =>
      _startingWrite ||
      (jobId != null &&
          !['complete', 'error', 'cancelled'].contains(progress?['phase']));
  bool get inputBusy => writing || inspecting || hashing;
  bool get checksumMatches =>
      checksum != null &&
      checksum!.toLowerCase() == expected.trim().toLowerCase();
  bool get capacityOk =>
      image != null && selected != null && selected!.size >= image!.size;
  bool get canWrite =>
      !inputBusy && capacityOk && (expected.trim().isEmpty || checksumMatches);
  void changed() {
    if (!_disposed) notifyListeners();
  }

  void report(Object failure) {
    error = failure.toString();
    Diagnostics.record(
      failure is CoreFailure ? 'core' : 'dart',
      failure is CoreFailure ? failure.kind : failure.runtimeType.toString(),
      level: 1000,
    );
    changed();
  }

  Future<void> initialize() async {
    await refresh();
    _refresh = Timer.periodic(const Duration(seconds: 8), (_) {
      if (!writing && !scanning) unawaited(refresh());
    });
  }

  Future<void> refresh() async {
    if (writing || scanning) return;
    final version = ++_scanVersion;
    scanning = true;
    changed();
    try {
      final response = await backend.call({'operation': 'devices'});
      if (_disposed || version != _scanVersion) return;
      devices = (response['devices'] as List)
          .map((d) => DeviceInfo.fromJson((d as Map).cast<String, dynamic>()))
          .toList();
      if (selected != null) {
        selected = devices
            .where(
              (d) =>
                  d.path == selected!.path && d.identity == selected!.identity,
            )
            .firstOrNull;
      }
    } catch (e) {
      report(e);
    } finally {
      if (version == _scanVersion) {
        scanning = false;
        changed();
      }
    }
  }

  Future<void> selectImage(String path) async {
    if (inputBusy) return;
    final version = ++_selectionVersion;
    inspecting = true;
    error = null;
    checksum = null;
    image = null;
    progress = null;
    changed();
    try {
      final response = await backend.call({
        'operation': 'inspect',
        'path': path,
      });
      if (!_disposed && version == _selectionVersion) {
        image = ImageInfo.fromJson(response);
      }
    } catch (e) {
      report(e);
    } finally {
      if (version == _selectionVersion) {
        inspecting = false;
        changed();
      }
    }
  }

  void clearImage() {
    if (inputBusy) return;
    ++_selectionVersion;
    image = null;
    checksum = null;
    error = null;
    changed();
  }

  void selectDevice(DeviceInfo device) {
    if (!inputBusy) {
      selected = device;
      changed();
    }
  }

  void setAlgorithm(String value) {
    if (!inputBusy) {
      algorithm = value;
      checksum = null;
      changed();
    }
  }

  void setExpected(String value) {
    expected = value;
    changed();
  }

  Future<void> calculate() async {
    if (inputBusy || image == null) return;
    final version = _selectionVersion;
    final file = image!;
    final algo = algorithm;
    hashing = true;
    checksum = null;
    error = null;
    changed();
    try {
      final result = await backend.call({
        'operation': 'checksum',
        'path': file.path,
        'algorithm': algo,
      });
      if (!_disposed && version == _selectionVersion && algo == algorithm) {
        checksum = result['checksum'] as String;
      }
    } catch (e) {
      report(e);
    } finally {
      hashing = false;
      changed();
    }
  }

  Future<void> write() async {
    if (!canWrite) return;
    error = null;
    _startingWrite = true;
    _cancelRequested = false;
    jobId = null;
    progress = {'phase': 'preparing'};
    changed();
    // Immutable operation options; preference edits cannot alter an active job.
    final request = {
      'source': image!.path,
      'device': selected!.json,
      'algorithm': algorithm,
      'expected_checksum': expected.trim().isEmpty ? null : expected.trim(),
      'verify': preferences.verify,
      'auto_eject': preferences.advanced && preferences.autoEject,
    };
    try {
      final result = await backend.call({
        'operation': 'write',
        'request': request,
      });
      jobId = result['id'] as int;
      if (_cancelRequested || _disposed) await cancel();
      if (_disposed) return;
      _poll = Timer.periodic(
        const Duration(milliseconds: 200),
        (_) => unawaited(poll()),
      );
      await poll();
    } catch (e) {
      report(e);
      progress = null;
    } finally {
      _startingWrite = false;
      changed();
    }
  }

  Future<void> poll() async {
    if (jobId == null || _polling || _disposed) return;
    _polling = true;
    try {
      progress = await backend.call({'operation': 'poll', 'id': jobId});
      final terminal = [
        'complete',
        'error',
        'cancelled',
      ].contains(progress!['phase']);
      if (terminal) {
        _poll?.cancel();
        if (progress!['error'] != null) {
          error = (progress!['error'] as Map)['message'] as String;
        }
      }
      changed();
    } catch (e) {
      report(e);
    } finally {
      _polling = false;
    }
  }

  Future<void> cancel() async {
    if (writing) {
      _cancelRequested = true;
      if (jobId == null) return;
      try {
        await backend.call({'operation': 'cancel', 'id': jobId});
      } catch (e) {
        report(e);
      }
    }
  }

  Future<void> eject() async {
    if (inputBusy || selected == null) return;
    try {
      await backend.call({'operation': 'eject', 'device': selected!.json});
      selected = null;
      await refresh();
    } catch (e) {
      report(e);
    }
  }

  void reset() {
    if (!writing) {
      jobId = null;
      progress = null;
      error = null;
      changed();
    }
  }

  @override
  void dispose() {
    if (writing) unawaited(cancel());
    _disposed = true;
    _refresh?.cancel();
    _poll?.cancel();
    super.dispose();
  }
}
