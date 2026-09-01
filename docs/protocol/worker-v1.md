# ClipLingo Worker Protocol v1

This document is the language-agnostic wire contract between the Rust shell and isolated inference worker.

## Goals

- small enough to implement identically in Rust and C++;
- explicit versioning and request correlation;
- bounded payloads;
- no dependency on Tauri, UI state, clipboard, or Windows selection details;
- transport-agnostic framing usable over Windows Named Pipes.

## Frame layout

All integer fields use little-endian encoding.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 4 | ASCII magic `CLNG` |
| 4 | 1 | protocol version (`1`) |
| 5 | 1 | message type |
| 6 | 8 | request ID (`u64`) |
| 14 | 4 | payload length (`u32`) |
| 18 | N | payload bytes |

Maximum payload size is **1 MiB**. Implementations must reject a larger declared or encoded payload before allocating/copying it.

A frame is exactly `18 + payload_length` bytes. Extra bytes are another frame at the transport layer, not part of the current frame.

## Message types

### `0x01` — TranslateRequest

Payload: UTF-8 source text.

The worker must return a response using the same request ID.

### `0x02` — TranslateResponse

Payload: UTF-8 translated text.

### `0x03` — ErrorResponse

Payload is exactly one byte containing the error code.

Current error codes:

- `0x01` — malformed request
- `0x02` — unsupported request
- `0x03` — translation failed
- `0x04` — worker unavailable

## Validation

A receiver must reject:

- invalid magic;
- unsupported protocol version;
- unknown message type;
- payload length above 1 MiB;
- truncated frame;
- trailing bytes when decoding one complete frame;
- invalid UTF-8 for request/translation payloads;
- error payload whose length is not exactly one byte;
- unknown error code.

## Privacy

The protocol carries selected and translated text. Normal logs and telemetry must never dump raw frame payloads or decoded text. Diagnostics may log request ID, message type, byte count, timing, state, and non-sensitive error codes.

## Compatibility

Protocol v1 is intentionally narrow. New fields that cannot be represented without ambiguity require a new protocol version or a newly defined message type; do not silently reinterpret existing payloads.
