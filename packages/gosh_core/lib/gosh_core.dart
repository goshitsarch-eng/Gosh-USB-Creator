import 'dart:convert';
import 'dart:ffi';
import 'dart:isolate';
import 'package:ffi/ffi.dart';

@Native<Pointer<Utf8> Function(Pointer<Uint8>, Size)>(symbol: 'gosh_call')
external Pointer<Utf8> _call(Pointer<Uint8> input, int length);
@Native<Void Function(Pointer<Utf8>)>(symbol: 'gosh_free')
external void _free(Pointer<Utf8> output);

class CoreFailure implements Exception {
  const CoreFailure(this.kind, this.message);
  final String kind;
  final String message;
  @override
  String toString() => message;
}

/// Owns the request allocation; Rust owns the response allocation. Each native
/// call runs on a worker isolate. No widget or Dart object is passed to Rust.
Map<String, dynamic> _invoke(Map<String, dynamic> request) {
  final bytes = utf8.encode(jsonEncode(request));
  if (bytes.length > 1024 * 1024) {
    throw const CoreFailure('validation', 'Request too large.');
  }
  final buffer = calloc<Uint8>(bytes.length);
  Pointer<Utf8>? response;
  try {
    buffer.asTypedList(bytes.length).setAll(0, bytes);
    response = _call(buffer, bytes.length);
    if (response == nullptr) {
      throw const CoreFailure('internal', 'No backend response.');
    }
    final envelope =
        jsonDecode(response.toDartString()) as Map<String, dynamic>;
    if (envelope['ok'] != true) {
      final error = envelope['error'] as Map<String, dynamic>;
      throw CoreFailure(error['kind'] as String, error['message'] as String);
    }
    return envelope['value'] as Map<String, dynamic>;
  } finally {
    if (response != null && response != nullptr) _free(response);
    calloc.free(buffer);
  }
}

abstract interface class Backend {
  Future<Map<String, dynamic>> call(Map<String, dynamic> request);
}

class RustBackend implements Backend {
  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) =>
      Isolate.run(() => _invoke(request));
}
