# Native iPhone controller build (0.3.4 build 3)

Target: physical `aarch64-apple-ios`, bundle ID `org.rerac.game.iospoc` unchanged.
The previous UIKit scene-lifecycle patch and all game/data/save systems remain in place.
No desktop release build or Intel gameplay acceptance is required by this workflow.

## Input path

Apple `GameController.controllers` / `GCExtendedGamepad` -> main-thread snapshot ->
Bevy raw gamepad events -> existing `input_map::sample_system` -> PS2 `PadFrame` ->
existing menu/front-end/gameplay code. Desktop input is unchanged. On iOS the gilrs
desktop-HID plugin is disabled to avoid duplicate device ownership. One selected
extended controller remains selected until it disconnects; discovery also handles
controllers attached before launch. Backgrounding releases input. Reconnection
gets a fresh native identity. No Backbone account, special entitlement, or private
API is used. Backbone's own system/app button is not remapped.

Xbox-labelled Backbone mapping: A=Cross/confirm/jump, B=Circle, X=Square, Y=Triangle;
Menu=Start/pause, Options=Select; shoulders=L1/R1, triggers=L2/R2, stick clicks=L3/R3;
d-pad and both sticks preserve ReRAC's existing pad mapping and deadzone/scale.
PlayStation-labelled controllers use the corresponding physical button positions.

## Validation boundaries

CI checks native Objective-C with the device SDK, builds and packages the real iPhone
executable, checks GameController/UIKit/Metal linkage and retains matching dSYM.
An explicit simulator-only self-test drives Apple's synthetic extended profile
through Bevy's real input processing and ReRAC's pad decoder: all 16 buttons,
hold/release edges, sticks/Y polarity, disconnect/reconnect and invalid float safety.
The ordinary no-data import-screen/Metal startup test still runs. Synthetic tests
are not physical Backbone, Start Game menu, audio, or gameplay acceptance.

## Physical acceptance

Re-sign the new IPA in Sideloadly with the same account and effective signed bundle
identifier as the installed app; preserve its Documents directory. The source bundle
identifier has not changed. Copy your own extracted files into Documents/ReRACData
while ReRAC is closed. Attach Backbone, launch, press Menu/Start when prompted,
navigate the front end with d-pad/stick, confirm Start Game with the south button,
then verify movement, camera, jump, attack, pause, unplug/replug and suspend/resume.
Existing rendering, audio, menus and saves have not been replaced.

Local bounded Documents/ReRACSupport/startup.log records GAMECONTROLLER_READY,
GAMECONTROLLER_CONNECTED/DISCONNECTED and only the first input per connection,
not per-button input history or device serial numbers. Keep the matching build dSYM
and a new .ips report if a crash occurs. No game assets are included in CI artifacts.

API references: Apple's GCExtendedGamepad, GCController.controllers and
GCSupportedGameControllers documentation. The native adapter is independently written.
