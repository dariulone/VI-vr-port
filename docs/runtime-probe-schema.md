# Runtime probe schema

This schema defines one bounded JSON record for a single game frame. The record is intended for local x64dbg or bridge logging after a translated process exists. It contains identifiers, counters, RVAs, resource handles, and small fixed-size vectors. It does not contain raw game buffers or extracted resources.

## Frame record

```json
{
  "frame_id": 0,
  "qpc_tick": 0,
  "phase": "mono",
  "camera": {
    "owner": "0x0",
    "output_rva": "0x322f8d0",
    "packet_copy_rva": "0x53159b0",
    "blocks": [null, null, null],
    "render_object": "0x0"
  },
  "frustum": {
    "prepare_rva": "0x4128b0",
    "shape_math_rva": "0x4440c60",
    "range_count": 0,
    "range_head": "0x0",
    "range_tail": "0x0"
  },
  "culling": {
    "dispatch_rva": "0x25d8590",
    "pass_id": 0,
    "index_count": 0,
    "submit_slot": "0xf0"
  },
  "queue": {
    "batch_count": 0,
    "batch_slot": "0x118",
    "plane_count": 0,
    "plane_slot": "0x100"
  },
  "presentation": {
    "slot_index": 0,
    "token": "0x0",
    "payload_bytes": 69888,
    "completion_count": 0,
    "completion_events": []
  },
  "pose": {
    "count": 0,
    "record_stride": 128,
    "metadata_flag_offset": 99,
    "aux_blocks": 4
  }
}
```

## Capture invariants

- `frame_id` and `qpc_tick` are monotonic within one process instance.
- `camera.blocks` contains three 16-byte blocks copied by `0x53159B0`.
- `queue.batch_count` never exceeds the statically observed 256-record temporary batch.
- `presentation.payload_bytes` is `69888` at the `+0x288` device call.
- `completion_events` records only the observed event ids `9` and `10` until runtime proves additional types.
- `pose.record_stride` is `0x80`, `pose.metadata_flag_offset` is `0x63`, and `pose.aux_blocks` is `4`.
- `culling.submit_slot` is `0xF0`, which is distinct from presentation plane slot `0x100` and resource batch slot `0x118`.

## Mono and stereo comparison

Capture the same stationary scene in `mono`, `left`, and `right` phases. The camera owner and gameplay state should remain stable. The eye packet, projection constants, culling range, plane resource, and completion token are compared by `frame_id`. A stereo frame is valid only when both eyes have distinct view or image handles and both reach completion before release.

Keep these records beside the private runtime trace. Commit derived aggregates or field names after review, not the raw capture.
