#!/bin/bash
set -euo pipefail
# Bevy 0.19.1's mobile example uses an ordinary Rust executable in an Xcode
# application target. Rust already linked the system frameworks; no C main,
# emulator, dylib bootstrap or replacement game loop is introduced.
test "${PLATFORM_NAME}" = iphoneos
test "${ARCHS}" = arm64
: "${RERAC_RUST_EXECUTABLE:?Build the device-target Rust executable first}"
test -x "$RERAC_RUST_EXECUTABLE"
test "$(lipo -archs "$RERAC_RUST_EXECUTABLE")" = arm64
mkdir -p "$TARGET_BUILD_DIR/$CONTENTS_FOLDER_PATH"
install -m 755 "$RERAC_RUST_EXECUTABLE" "$TARGET_BUILD_DIR/$EXECUTABLE_PATH"
