class ImageInfo {
  ImageInfo.fromJson(Map<String, dynamic> json)
    : path = json['path'] as String,
      name = json['name'] as String,
      size = json['size'] as int,
      format = json['format'] as String,
      recognized = json['recognized'] as bool,
      warnings = (json['warnings'] as List).cast<String>();
  final String path, name, format;
  final int size;
  final bool recognized;
  final List<String> warnings;
}

class DeviceInfo {
  DeviceInfo.fromJson(this.json)
    : path = json['path'] as String,
      name = json['name'] as String,
      size = json['size'] as int,
      identity = json['identity'] as String,
      mounts = (json['mount_points'] as List).cast<String>();
  final Map<String, dynamic> json;
  final String path, name, identity;
  final int size;
  final List<String> mounts;
}

String sizeLabel(num bytes) {
  const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
  var value = bytes.toDouble();
  var unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return '${value.toStringAsFixed(unit == 0 ? 0 : 1)} ${units[unit]}';
}
