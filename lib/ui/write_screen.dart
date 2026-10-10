import 'package:desktop_drop/desktop_drop.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../controller.dart';
import '../models.dart';

class Section extends StatelessWidget {
  const Section(this.title, this.child, {super.key});
  final String title;
  final Widget child;
  @override
  Widget build(BuildContext context) => Card(
    child: Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(title, style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 16),
          child,
        ],
      ),
    ),
  );
}

class WriteScreen extends StatelessWidget {
  const WriteScreen({
    super.key,
    required this.controller,
    required this.open,
    required this.write,
  });
  final AppController controller;
  final VoidCallback open, write;
  @override
  Widget build(BuildContext context) {
    final c = controller;
    final image = c.image;
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        Text(
          'Write an image to USB',
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 6),
        const Text('Select your image, check your target, then write.'),
        const SizedBox(height: 24),
        Section(
          'Source image',
          DropTarget(
            enable: !c.inputBusy,
            onDragDone: (detail) {
              if (detail.files.isNotEmpty) {
                c.selectImage(detail.files.first.path);
              }
            },
            child: image == null
                ? OutlinedButton.icon(
                    onPressed: c.inputBusy ? null : open,
                    icon: const Icon(Icons.add_circle_outline),
                    label: Padding(
                      padding: const EdgeInsets.symmetric(vertical: 28),
                      child: Text(
                        c.inspecting
                            ? 'Inspecting image…'
                            : 'Choose ISO / IMG or drop a file here',
                      ),
                    ),
                  )
                : Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Row(
                        children: [
                          const Icon(Icons.insert_drive_file_outlined),
                          const SizedBox(width: 12),
                          Expanded(
                            child: SelectableText(
                              image.name,
                              style: Theme.of(context).textTheme.titleSmall,
                            ),
                          ),
                          const SizedBox(width: 8),
                          Text(sizeLabel(image.size)),
                          IconButton(
                            tooltip: 'Clear source image',
                            onPressed: c.inputBusy ? null : c.clearImage,
                            icon: const Icon(Icons.close),
                          ),
                        ],
                      ),
                      const SizedBox(height: 6),
                      SelectableText(
                        image.path,
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                      if (c.preferences.advanced) ...[
                        const Divider(height: 28),
                        Row(
                          children: [
                            Icon(
                              image.recognized
                                  ? Icons.info_outline
                                  : Icons.warning_amber_outlined,
                              size: 18,
                            ),
                            const SizedBox(width: 8),
                            TextButton(
                              onPressed: () => showDialog<void>(
                                context: context,
                                builder: (context) => AlertDialog(
                                  title: Text(image.format),
                                  content: const Text(
                                    'ISO 9660 identifies optical disc data. MBR and GPT identify disk layouts. Signature detection does not establish bootability or trust. Always use an image from a trusted source.',
                                  ),
                                  actions: [
                                    TextButton(
                                      onPressed: () => Navigator.pop(context),
                                      child: const Text('Close'),
                                    ),
                                  ],
                                ),
                              ),
                              child: Text(image.format),
                            ),
                          ],
                        ),
                      ],
                      ...image.warnings.map(
                        (w) => Padding(
                          padding: const EdgeInsets.only(top: 4),
                          child: Text(w),
                        ),
                      ),
                      if (c.selected != null && !c.capacityOk)
                        Text(
                          'Image exceeds the selected drive’s capacity.',
                          style: TextStyle(
                            color: Theme.of(context).colorScheme.error,
                          ),
                        ),
                    ],
                  ),
          ),
        ),
        const SizedBox(height: 16),
        if (image != null) ...[
          Section(
            'Verify checksum',
            Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Row(
                  children: [
                    DropdownButton<String>(
                      value: c.algorithm,
                      onChanged: c.inputBusy ? null : (v) => c.setAlgorithm(v!),
                      items: const [
                        DropdownMenuItem(
                          value: 'sha256',
                          child: Text('SHA-256'),
                        ),
                        DropdownMenuItem(value: 'md5', child: Text('MD5')),
                      ],
                    ),
                    const SizedBox(width: 16),
                    OutlinedButton(
                      onPressed: c.inputBusy ? null : c.calculate,
                      child: Text(c.hashing ? 'Calculating…' : 'Calculate'),
                    ),
                    if (c.hashing)
                      const Padding(
                        padding: EdgeInsets.only(left: 16),
                        child: SizedBox.square(
                          dimension: 18,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        ),
                      ),
                  ],
                ),
                if (c.checksum != null) ...[
                  const SizedBox(height: 12),
                  Row(
                    children: [
                      Expanded(
                        child: SelectableText(
                          c.checksum!,
                          style: const TextStyle(fontFamily: 'monospace'),
                        ),
                      ),
                      IconButton(
                        tooltip: 'Copy checksum',
                        onPressed: () async {
                          await Clipboard.setData(
                            ClipboardData(text: c.checksum!),
                          );
                          if (context.mounted) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(content: Text('Checksum copied.')),
                            );
                          }
                        },
                        icon: const Icon(Icons.copy_outlined),
                      ),
                    ],
                  ),
                ],
                const SizedBox(height: 16),
                TextFormField(
                  initialValue: c.expected,
                  enabled: !c.writing,
                  onChanged: c.setExpected,
                  decoration: InputDecoration(
                    labelText: 'Expected checksum (optional)',
                    helperText: c.expected.trim().isEmpty
                        ? 'MD5 checks integrity; SHA-256 is preferred.'
                        : c.checksum == null
                        ? 'Calculate to compare before writing.'
                        : c.checksumMatches
                        ? 'Checksums match'
                        : 'Checksums do not match — writing is disabled.',
                    errorText:
                        c.expected.isNotEmpty &&
                            !RegExp(
                              '^[a-fA-F0-9]{${c.algorithm == 'sha256' ? 64 : 32}}\$',
                            ).hasMatch(c.expected.trim())
                        ? 'Enter a ${c.algorithm == 'sha256' ? 64 : 32}-digit hexadecimal checksum.'
                        : null,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 16),
        ],
        Section(
          'Target device',
          Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Row(
                children: [
                  Expanded(
                    child: Text(
                      '${c.devices.length} removable device${c.devices.length == 1 ? '' : 's'} found',
                    ),
                  ),
                  OutlinedButton.icon(
                    onPressed: c.scanning || c.writing ? null : c.refresh,
                    icon: const Icon(Icons.refresh),
                    label: Text(c.scanning ? 'Scanning…' : 'Refresh'),
                  ),
                ],
              ),
              if (c.devices.isEmpty)
                const Padding(
                  padding: EdgeInsets.symmetric(vertical: 24),
                  child: Text(
                    'Insert a removable USB drive, then refresh. System disks are excluded.',
                  ),
                ),
              ...c.devices.map(
                (d) => Padding(
                  padding: const EdgeInsets.only(top: 12),
                  child: Card(
                    color: c.selected?.identity == d.identity
                        ? Theme.of(context).colorScheme.secondaryContainer
                        : null,
                    child: ListTile(
                      enabled: !c.inputBusy,
                      onTap: () => c.selectDevice(d),
                      leading: Icon(
                        c.selected?.identity == d.identity
                            ? Icons.radio_button_checked
                            : Icons.radio_button_off,
                      ),
                      title: Text(d.name),
                      trailing: Text(sizeLabel(d.size)),
                      subtitle: Text(
                        '${d.path}${d.mounts.isEmpty ? '' : '\nMounted: ${d.mounts.join(', ')}'}',
                      ),
                    ),
                  ),
                ),
              ),
              if (c.selected != null)
                Align(
                  alignment: Alignment.centerRight,
                  child: TextButton.icon(
                    onPressed: c.inputBusy ? null : c.eject,
                    icon: const Icon(Icons.eject_outlined),
                    label: const Text('Eject selected drive'),
                  ),
                ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        if (c.progress != null) ...[
          Section('Operation', _Progress(controller: c)),
          const SizedBox(height: 16),
        ],
        Section(
          'Write options',
          Wrap(
            alignment: WrapAlignment.spaceBetween,
            crossAxisAlignment: WrapCrossAlignment.center,
            spacing: 20,
            runSpacing: 12,
            children: [
              SizedBox(
                width: 260,
                child: CheckboxListTile(
                  contentPadding: EdgeInsets.zero,
                  controlAffinity: ListTileControlAffinity.leading,
                  value: c.preferences.verify,
                  onChanged: c.inputBusy
                      ? null
                      : (v) async {
                          try {
                            await c.preferences.setBool('verify', v!);
                            c.changed();
                          } catch (e) {
                            c.report(e);
                          }
                        },
                  title: const Text('Verify after writing'),
                ),
              ),
              FilledButton.icon(
                onPressed: c.canWrite ? write : null,
                icon: const Icon(Icons.usb),
                label: const Padding(
                  padding: EdgeInsets.symmetric(vertical: 12),
                  child: Text('Write image'),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 12),
        Text(
          'Writing erases all data on the selected drive.',
          style: Theme.of(context).textTheme.bodySmall,
        ),
      ],
    );
  }
}

class _Progress extends StatelessWidget {
  const _Progress({required this.controller});
  final AppController controller;
  @override
  Widget build(BuildContext context) {
    final c = controller;
    final p = c.progress!;
    final total = p['total'] as int? ?? 0;
    final bytes = p['bytes'] as int? ?? 0;
    final phase = p['phase'] as String;
    final fraction = total == 0 ? null : (bytes / total).clamp(0.0, 1.0);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(switch (phase) {
          'complete' => 'Image written successfully',
          'cancelled' => 'Cancelled — target may be incomplete',
          'error' => 'Write failed',
          'verifying' => 'Verifying written bytes…',
          'writing' => 'Writing to USB…',
          _ => 'Preparing device…',
        }),
        if (c.writing) ...[
          const SizedBox(height: 16),
          LinearProgressIndicator(value: fraction),
          const SizedBox(height: 10),
          Text(
            '${fraction == null ? '' : '${(fraction * 100).round()}% · '}${sizeLabel(bytes)} / ${sizeLabel(total)} · ${sizeLabel(p['speed_bps'] as int? ?? 0)}/s · ETA ${p['eta_seconds'] ?? 0}s',
          ),
          Align(
            alignment: Alignment.centerRight,
            child: TextButton(
              onPressed: c.cancel,
              child: const Text('Cancel operation'),
            ),
          ),
        ] else
          Align(
            alignment: Alignment.centerRight,
            child: TextButton(
              onPressed: c.reset,
              child: Text(phase == 'complete' ? 'Write another' : 'Dismiss'),
            ),
          ),
        ...((p['warnings'] as List?) ?? []).map(
          (w) => Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Text(w.toString()),
          ),
        ),
      ],
    );
  }
}
