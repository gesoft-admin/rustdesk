// Modified by Gesoft, 2026-10-02. See CHANGES-GESOFT.md.
import 'dart:io';

import 'package:flutter/painting.dart';

String? name() => Platform.environment['RUSTDESK_BRAND_NAME'];
String? accent() => Platform.environment['RUSTDESK_BRAND_ACCENT'];

ImageProvider? image(String asset) {
  final dir = Platform.environment['RUSTDESK_BRAND_DIR'];
  if (dir == null || dir.isEmpty) return null;
  final file = File('$dir${Platform.pathSeparator}$asset.png');
  return file.existsSync() ? FileImage(file) : null;
}
