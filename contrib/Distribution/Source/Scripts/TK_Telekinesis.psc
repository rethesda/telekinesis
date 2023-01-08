scriptName TK_Telekinesis hidden

; Sets up a new connection and starts scanning for devices. This will 
; automatically connect to every single bluetooth toy Buttplug.io knows about.
; This will find any device that is in-reach and coupled with your PCs bluetooth 
; adapater. Right now the scanning will continue indefinitely, so new
; devices might be added at any point in time automatically
; Returns false when the command could not be sent
Bool function TK_ScanForDevices() global native

; Vibrate all devices that are currently connected.
; Speed is any float between 0.0(=off) and 1.0 (=full power)
; TK_VibrateAll( 0 ) should also be used for stopping the vibration,
; as it provides a smoother experience than Tk_StopAll
; Returns false when the command could not be sent
Bool function TK_VibrateAll(Float speed) global native

; Immediately stops all connected devices. This should be used for
; shutdown, before calling Tk_Close to assure that everything stopped.
;
; NOTE: You could also use it to stop device vibration manually, but I've
; experienced that it will cause weird behavior: Some devices still store
; the last vibration speed
; Returns false when the command could not be sent
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
; usable from this point on. However, you may run TK_ScanForDevices to
; start over again.
Bool function Tk_Close() global native
