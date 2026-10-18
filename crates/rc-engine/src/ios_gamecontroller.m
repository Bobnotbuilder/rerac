// ReRAC iOS adapter, independently written against Apple's public GameController API.
// Existing gamepad-to-libpad2 conversion remains in input_map.rs. No private APIs.
#import <Foundation/Foundation.h>
#import <GameController/GameController.h>
#import <UIKit/UIKit.h>
#import <TargetConditionals.h>
#include <stdint.h>
#include <string.h>

typedef struct {
    uint64_t identity;
    float axes[4]; // left x/y, right x/y; Apple/Bevy positive Y is up
    float buttons[16]; // order matches ios_gamepad::BUTTONS
} RRControllerSnapshot;
_Static_assert(sizeof(RRControllerSnapshot) == 88, "Rust/C snapshot layout mismatch");

static void snapshot(GCExtendedGamepad *pad, uint64_t identity, RRControllerSnapshot *out) {
    out->identity = identity;
    out->axes[0] = pad.leftThumbstick.xAxis.value;
    out->axes[1] = pad.leftThumbstick.yAxis.value;
    out->axes[2] = pad.rightThumbstick.xAxis.value;
    out->axes[3] = pad.rightThumbstick.yAxis.value;
    GCControllerButtonInput *inputs[16] = {
        pad.buttonA, pad.buttonB, pad.buttonX, pad.buttonY,
        pad.leftShoulder, pad.rightShoulder, pad.leftTrigger, pad.rightTrigger,
        pad.buttonMenu, pad.buttonOptions, pad.leftThumbstickButton, pad.rightThumbstickButton,
        pad.dpad.up, pad.dpad.down, pad.dpad.left, pad.dpad.right
    };
    for (int i = 0; i < 16; ++i) out->buttons[i] = inputs[i].value; // optional nil => 0
}

// Called once per Bevy PreUpdate on the AppKit/UIKit main thread, never an async task.
int rr_gc_poll(RRControllerSnapshot *out) {
    memset(out, 0, sizeof(*out));
    if (![NSThread isMainThread]) return -1;
    @autoreleasepool {
        static GCController *selected;
        static uint64_t serial = 0;
        NSArray<GCController *> *controllers = GCController.controllers;
        if (selected && ![controllers containsObject:selected]) selected = nil;
        if (!selected) {
            for (GCController *candidate in controllers) {
                if (candidate.extendedGamepad) {
                    selected = candidate; // retain selection until disconnect; no per-frame switching
                    ++serial;
                    // Route menu/options to the game rather than a system double-press gesture.
                    candidate.extendedGamepad.buttonMenu.preferredSystemGestureState = GCSystemGestureStateDisabled;
                    candidate.extendedGamepad.buttonOptions.preferredSystemGestureState = GCSystemGestureStateDisabled;
                    break;
                }
            }
        }
        if (!selected) return 0;
        out->identity = serial;
        // Background/inactive frames must release all held controls, not leave Ratchet moving.
        if (UIApplication.sharedApplication.applicationState != UIApplicationStateActive) return 1;
        snapshot(selected.extendedGamepad, serial, out);
        return 1;
    }
}

// Synthetic controller for CI only. This is NOT physical Backbone acceptance.
// Device binaries have no synthetic controller implementation.
#if TARGET_OS_SIMULATOR
int rr_gc_simulated_snapshot(uint32_t mask, const float axes[4], RRControllerSnapshot *out) {
    if (![NSThread isMainThread]) return -1;
    @autoreleasepool {
        GCController *controller = [GCController controllerWithExtendedGamepad];
        GCExtendedGamepad *pad = controller.extendedGamepad;
        [pad.leftThumbstick.xAxis setValue:axes[0]];
        [pad.leftThumbstick.yAxis setValue:axes[1]];
        [pad.rightThumbstick.xAxis setValue:axes[2]];
        [pad.rightThumbstick.yAxis setValue:axes[3]];
        GCControllerButtonInput *inputs[16] = {
            pad.buttonA, pad.buttonB, pad.buttonX, pad.buttonY,
            pad.leftShoulder, pad.rightShoulder, pad.leftTrigger, pad.rightTrigger,
            pad.buttonMenu, pad.buttonOptions, pad.leftThumbstickButton, pad.rightThumbstickButton,
            pad.dpad.up, pad.dpad.down, pad.dpad.left, pad.dpad.right
        };
        // Snapshot D-pad directions share axes. Writing Up then Down=0 clears
        // Up on current iOS; set both axes atomically after ordinary buttons.
        // https://developer.apple.com/documentation/gamecontroller/gccontrollerdirectionpad/setvalueforxaxis(_:yaxis:)
        for (int i = 0; i < 12; ++i) [inputs[i] setValue:(mask & (1u << i)) ? 1.0f : 0.0f];
        const float dx = !!(mask & (1u << 15)) - !!(mask & (1u << 14));
        const float dy = !!(mask & (1u << 12)) - !!(mask & (1u << 13));
        [pad.dpad setValueForXAxis:dx yAxis:dy];
        memset(out, 0, sizeof(*out));
        snapshot(pad, 1, out);
        return 1;
    }
}
#endif
