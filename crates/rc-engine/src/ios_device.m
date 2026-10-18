// Local-only iPhone layout/health measurements. Public UIKit/Foundation/Mach APIs.
#import <Foundation/Foundation.h>
#import <UIKit/UIKit.h>
#import <mach/mach.h>
#include <stdint.h>
#include <string.h>

typedef struct {
    float safe[4]; // fractions, left/top/right/bottom; stable across Retina scale
    int32_t active;
    int32_t thermal;
    uint64_t footprint;
    float battery; // -1 = unavailable; fraction is NOT an energy measurement
} RRDeviceInfo;
_Static_assert(sizeof(RRDeviceInfo) == 40, "Rust/C layout mismatch");

int rr_ios_info(RRDeviceInfo *out, int expensive) {
    memset(out, 0, sizeof(*out));
    out->battery = -1;
    if (![NSThread isMainThread]) return -1;
    @autoreleasepool {
        UIApplication *app = UIApplication.sharedApplication;
        out->active = app.applicationState == UIApplicationStateActive;
        out->thermal = (int32_t)NSProcessInfo.processInfo.thermalState;
        UIWindow *window = nil;
        for (UIScene *scene in app.connectedScenes) {
            if (![scene isKindOfClass:UIWindowScene.class]) continue;
            for (UIWindow *candidate in ((UIWindowScene *)scene).windows) {
                if (candidate.isKeyWindow && !candidate.hidden) { window = candidate; break; }
            }
            if (window) break;
        }
        UIView *view = window.rootViewController.view;
        if (!view || view.bounds.size.width < 1 || view.bounds.size.height < 1) return 0;
        UIEdgeInsets s = view.safeAreaInsets;
        // Four points of breathing room, symmetrised by the Rust UI policy.
        float w = (float)view.bounds.size.width, h = (float)view.bounds.size.height;
        out->safe[0] = (s.left + 4) / w; out->safe[1] = (s.top + 4) / h;
        out->safe[2] = (s.right + 4) / w; out->safe[3] = (s.bottom + 4) / h;
        if (expensive) {
            task_vm_info_data_t info;
            mach_msg_type_number_t count = TASK_VM_INFO_COUNT;
            if (task_info(mach_task_self(), TASK_VM_INFO, (task_info_t)&info, &count) == KERN_SUCCESS)
                out->footprint = info.phys_footprint;
            UIDevice *device = UIDevice.currentDevice;
            device.batteryMonitoringEnabled = YES;
            out->battery = device.batteryLevel;
        }
        return 1;
    }
}
