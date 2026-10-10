import 'dart:developer' as developer;
import 'dart:io';
import 'package:path/path.dart' as path;
import 'package:path_provider/path_provider.dart';

/// One diagnostic stream for presentation, bridge and backend failures.
/// Only categories are persisted; detailed errors remain in the visible UI.
class Diagnostics {
  static File? _file;
  static Future<void> _pending = Future.value();
  static Future<void> initialize() async {
    final directory = await getApplicationSupportDirectory();
    await directory.create(recursive: true);
    _file = File(path.join(directory.path, 'diagnostics.log'));
  }

  static void record(String origin, String category, {int level = 800}) {
    developer.log(category, name: 'gosh.$origin', level: level);
    final file = _file;
    if (file == null) return;
    _pending = _pending
        .then((_) async {
          if (await file.exists() && await file.length() > 1024 * 1024) {
            final old = File('${file.path}.previous');
            if (await old.exists()) await old.delete();
            await file.rename(old.path);
          }
          await file.writeAsString(
            '${DateTime.now().toUtc().toIso8601String()} $origin $level $category\n',
            mode: FileMode.append,
          );
        })
        .catchError((Object error) {
          developer.log(
            'Diagnostic write failed',
            name: 'gosh.logging',
            level: 900,
          );
        });
  }
}
