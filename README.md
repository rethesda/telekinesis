# Telekinesis (Bluetooth Toy Control for Skyrim) 1.1.0

Telekinesis is a mod that brings native support for naughty devices (bluetooth or others) to Skyrim SE, AE.

## Features

- Sync real toys with in-game toy events from `Devious Devices`, `Toys & Love` and more
- Control toys during `Sexlab` or `Toys & Love` animations
- Associate toys with certain body parts for a more immersive experience
- Written as a native SKSE plugin for minimal latency and low setup effort (works natively, no background processes required)
- Re-usable API for mod authors

Watch the demo video:

<a href="https://youtu.be/XrXUIUjuSRQ?t=119" title="Video Tutorial">
  <img src="doc/prev.png" width=350 />
</a>

## 1. Installation

1. Install `Telekinesis.7z` with a mod manager
2. Install dependencies `SKSE64`, `SkyUI`, `Address Library for SKSE Plugins`
3. Install optional dependencies `Sexlab`, `Sexlab Arousal`, `Toys & Love`

**Conflicts**: Applications that access the same bluetooth devices at the same time

## 2. Quickstart

1. Connect a bluetooth toy in your operating system
2. Once connected, toys will show up in your in-game notifications (and in the MCM device page)
3. Open the MCM Page `Devices` and **enable** the connected device for usage (See [Manual: 2. Device Setup](./doc/Manual.md#Device_Setup))
4. Configure the emergency stop hotkey (default `DEL`)
5. Open the MCM and enable functionalities of your choice
6. [PLEASE READ THE MANUAL](./doc/Manual.md#Integration)

**Video guide**

<a href="https://youtu.be/XrXUIUjuSRQ" title="Video Tutorial">
  <img src="doc/prev1.png" width="400"/>
</a>


## 3. FAQ

### Limitations & Support

**Working Game Versions**
 *    Skyrim SE (v1.5.97.0)
 *    Skyrim AE (1.6.640.0) and (1.6.1130.0), all AE versions should work.
   
**Limited Support**
 * Skyrim VR (Certain MCM Inputs seem broken, try **Telekinesis.json** for configuring devices as a workaround)

**Unsupported**
 * Skyrim LE

I do not own Skyrim VR or Skyrim LE and won't be able to test it.

**Limitations**
 * Only supports vibrating devices (for now)
 * [List of toys that might work](https://iostindex.com/?filter0ButtplugSupport=4&filter1Connection=Bluetooth%204%20LE,Bluetooth%202&filter2Features=OutputsVibrators)


### Migrating from old versions

- If you come from 1.0.0 (Beta), your settings will be dropped
- Uninstall `TelekinesisTest.esp` and delete it forever (it won't be needed again)
- Migrating from the early alpha versions while staying on the same save is unsupported, start a new game, or try to fix on your own.

### Devices don't connect

Check that:

1. Your device is coupled correctly (bluetooth settings)
2. Your device has enough battery
3. Your device is supported by buttplug.io, see [List of toys that might work](https://iostindex.com/?filter0ButtplugSupport=4&filter1Connection=Bluetooth%204%20LE,Bluetooth%202&filter2Features=OutputsVibrators)
4. Test it with [Intiface Central Desktop App](https://intiface.com/central), if a vibrator works in that app, and not in this plugin, its an issue with the mod.
5. The device is Bluetooth. Devices that use serial port, lovesense connect, or other exocit connection mechanisms can work with Intiface App if your enable those connection methods (Server Settings) and select Intiface-Webapp as the connection method in Telekinesis MCM.

### Devices don't vibrate

1. Make sure that your device is enabled in Page `Devices`
2. Make sure it has full battery (with low battery it might still be able to connect but not move)

### Why?

I know that there was already an amazing solution with **G.I.F.T**, but sunk cost fallacy (and the prospect of TES 6 being released something like 2030) drove me to continue with my own little approach, and I think I managed to create a really fast and easy to use solution.

### Bug Reports

If anything fails or behaves in an unexpected way, include the Papyrus logs `Pyprus.0.log` and the Logs of this plugin (`%USERPROFILE%/My Games/Sykrim Special Edition/SKSE/Telekinesis.log`)
* If you can reproduce the issue, adapt the debug level by changing `Telekinesis.json` and set everything to `Trace`.

## License

This mod is free software and can be used under the terms of the [Apache License V2](LICENSE) 

## Changelog

## 1.1.0

- Migrating from Beta will reset your MCM settings

- Add support for funscript patterns
  * Only works with vibrator files `vibration.funscript` files for now
  * Other patterns are still being displayed

- Add support for events (device tags)
  * This allows associating devices with certain events that correlate to body parts (see manual)

- Improve integration for Sexlab, Devious Devices, Toys & Love:
  * Introduced a lot of new generic vibration options that are available for almost all of the vibration events
    * Strength can be regulated linearly or with a funscript pattern
    * Use random patterns
    * Support matching devices with events (body parts)

  * Devious Devices
    * Uses actual DD vibration strength (device vibrated strongly, very strongly etc.) instead of a random speed value.
    * Tag/Event support to match equipped dd stimulation devices with body parts (Nipple, Anal, Vaginal)

  * Sexlab
    * Match devices with animation tags
    * Control Strength through sexlab arousal
    * Support for denial

  * Toys&Love
    * Match with animation tags
    * Control strength through rousing
    * Support Denial, Body Part Penetration and Fondling events

  * Skyrim Chain Beast
    * Support Gemmed Beast Vibrations (`SCB_VibeEvent`)
    * Disclaimer: Seems to not work with Chainbeasts v7.0.0, unless SCB_VibeEffect.psc is recompiled
      from source and the psx was replaced in Script folder

- Technical Improvements
  * Add support for simultaneous and overlapping vibration events and patterns. 
    * Previously every new device action aborted all running tasks
    * Technical requirement for long running patterns and to assure a seamless
      experience with mods that do a lof of different things at the same time.
    * Papyrus API had to be reworked to use task handles
  * WebSocket (Intiface) connection now works
    * This allows to use Intiface App as backend control instead of the default in-process backend

## 1.0.0

- Complete rework of everything
- Devious Devices Integration
- Toys & Love Integration
- Sexlab integration
- Add emergency stop hotkey

## 0.3.0

**Features**:

- Add `Tele.VibrateAllFor` to vibrate for a specific duration and then stop
- Reworked/broke entire API
    - Vibration speed is now value between 0 and 100
    - Shorter functions i.e. `Tele.VibrateAll` instead of `Tk_Telekinesis.Tk_VibrateAll`

**Fixes**:
- Now loads on AE (as intended)
- More stability/stutter fixes
    - Not a single possibly blocking call left in papyrus thread
    - Actually link against updated rust lib, so the fix from 0.2.0 is now correctly included

## 0.2.0

- Support message queuing to reduce mini lags
- More consistent naming of API functions

## 0.1.0

- Initial Version
