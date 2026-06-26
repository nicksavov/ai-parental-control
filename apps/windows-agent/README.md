# windows-agent

The child agent for Windows 11. C# / .NET 8, runs as a Windows Service (SYSTEM). Open-by-default platform, so it ships in v0 alongside Android and de-risks the cross-platform protocol.

## v0 scope

Pairing, DNS filtering (DNS hook / WFP), per-process usage tracking (GetForegroundWindow plus ETW plus Job Objects), app blocking (Job Objects plus AppLocker), daily-total plus bedtime session lock.

## Notes

- There is no Microsoft Family Safety screen-time API, so usage tracking is built from ETW and Job Objects, not an MS data source.
- Deep HTTPS filtering needs TLS interception, which we do not do. Stay at DNS level.
- Tamper resistance: Windows Service as SYSTEM plus AppLocker blocking uninstall tools for the child account. A child with admin or physical access can still bypass; that is surfaced as a parent alert.
- Distribution: code-sign (OV cert or MSIX via Store re-signing) to limit SmartScreen friction and avoid a PUA flag. Keep the UI visible and the uninstall path clear for the parent.
- On-device AI via ONNX Runtime plus DirectML.
