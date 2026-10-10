import 'dart:io';
import 'package:code_assets/code_assets.dart';
import 'package:hooks/hooks.dart';

void main(List<String> args) async {
  await build(args, (input, output) async {
    if (!input.config.buildCodeAssets) return;
    final os = input.config.code.targetOS;
    final arch = input.config.code.targetArchitecture;
    final target = switch ((os, arch)) {
      (OS.linux, Architecture.x64) => 'x86_64-unknown-linux-gnu',
      (OS.linux, Architecture.arm64) => 'aarch64-unknown-linux-gnu',
      (OS.macOS, Architecture.x64) => 'x86_64-apple-darwin',
      (OS.macOS, Architecture.arm64) => 'aarch64-apple-darwin',
      (OS.windows, Architecture.x64) => 'x86_64-pc-windows-msvc',
      (OS.windows, Architecture.arm64) => 'aarch64-pc-windows-msvc',
      _ => throw UnsupportedError('Unsupported desktop target: $os/$arch'),
    };
    final rust = input.packageRoot.resolve('rust/');
    final result = await Process.run('cargo', [
      'build',
      '--locked',
      '--release',
      '--target',
      target,
      '--target-dir',
      input.outputDirectory.resolve('target/').toFilePath(),
    ], workingDirectory: rust.toFilePath());
    if (result.exitCode != 0) {
      throw StateError(
        'Rust build failed (${result.exitCode})\n${result.stderr}',
      );
    }
    final name = os == OS.windows
        ? 'gosh_core.dll'
        : os == OS.macOS
        ? 'libgosh_core.dylib'
        : 'libgosh_core.so';
    output.assets.code.add(
      CodeAsset(
        package: input.packageName,
        name: 'gosh_core.dart',
        linkMode: DynamicLoadingBundled(),
        file: input.outputDirectory.resolve('target/$target/release/$name'),
      ),
    );
    output.dependencies.addAll([
      rust.resolve('Cargo.toml'),
      rust.resolve('Cargo.lock'),
      ...Directory.fromUri(
        rust.resolve('src/'),
      ).listSync(recursive: true).whereType<File>().map((f) => f.uri),
    ]);
  });
}
