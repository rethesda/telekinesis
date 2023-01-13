scriptName TK_Telekinesis hidden

; Sets up a new connection and starts scanning for devices. This will 
; automatically connect to every single bluetooth toy Buttplug.io knows about.
; This will find any device that is in-reach and coupled with your PCs bluetooth 
; adapater. Right now the scanning will continue indefinitely, so new
; devices might be added at any point in time automatically
Bool function TK_ScanForDevices() global native

; Vibrate all devices that are currently connected (until stopped manually).
; Speed is any float between 0.0(=off) and 1.0(=full power)
; `TK_VibrateAll(0)` should also be used for stopping the vibration,
; as it provides a smoother experience than Tk_StopAll
Bool function TK_VibrateAll(Float speed) global native

; Vibrate all devices that are currently connected for `duration_sec` seconds
; Calls to `TK_VibrateAll` or `TK_VibrateAllFor` that happen before `duration_sec` 
; has ended will owerwrite `speed` and `duration_sec` to the new calls value.
Bool function TK_VibrateAllFor(Float speed, Float duration_sec) global native

; Immediately stops all connected devices. This can be used for
; shutdown of ALL device actions before calling Tk_Close to assure that
; everything stopped.
Bool function Tk_StopAll() global native

; Returns a stream of messages that describe events in Tk
; - RETURN a string describing the Event or an empty Array if nothing happened
; Examples messages:
;  * "Device XY connected" (This device is connected and will be controlled)
;  * "Device XY disconnected" (This device is no longer connected and will be ignored)
;  * "Vibrating X devices..." (A vibrate command was successful and vibrated X devices)
; When multiple Mods consume this, they will steal each others events
String[] function Tk_PollEvents() global native

; Close the connection and dispose all structures. Telekinesis will not be
; usable from this point on. However, you may run TK_ScanForDevices to create
; a new connection and start over again.
Bool function Tk_Close() global native

; Note:
; All functions return false when the command could not be sent
; This is likely due to the input queue being full, which should
; virtually never happen, unless you spam commands or something is very wrong
; In all cases you should check the error log
