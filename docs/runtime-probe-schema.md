# Runtime probe schema

This schema defines one bounded JSON record for a single game frame. The record is intended for local x64dbg or bridge logging after a translated process exists. It contains identifiers, counters, RVAs, resource handles, and small fixed-size vectors. It does not contain raw game buffers or extracted resources.

The safe Rust representation is `gta6-vr-core::RuntimeProbeFrame`. Its validator accepts a pre-present frame with payload `0` and a completed presentation frame with payload `69888`.

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
  },
  "input": {
    "gameinput_create_rva": "0x5f3fed0",
    "wrapper_rva": "0x59258a0",
    "interface_ptr": "0x0",
    "vtable_ptr": "0x0",
    "trigger_edge": 0
  },
  "weapon": {
    "event_dispatch_rva": "0x5f34810",
    "action_id": -1,
    "phase": -1,
    "weapon_entity": "0x0",
    "state_flags": 0,
    "resource_ptr": "0x0"
  },
  "physics": {
    "vehicle_deform": "0x0",
    "explosion_vehicle_buffer": "0x0",
    "explosion_ped_buffer": "0x0",
    "particle_collision_job": "0x0",
    "rain_resource": "0x0",
    "water_mode": 0,
    "constraint_index": -1,
    "body": {
      "shape_batch_rva": "0x645740",
      "aggregate_rva": "0x4f5b770",
      "shape_property_rva": "0x2687a20",
      "weighted_center_rva": "0x55b1160",
      "inertia_rva": "0x55b1200",
      "property_initializer_rva": "0x6d9ff0",
      "dynamic_update_rva": "0x1f97990",
      "body_ptr": "0x0",
      "body_type": 0,
      "state_flags": 0,
      "shape_count": 0,
      "mass": 0.0,
      "inverse_mass": 0.0,
      "state_index": -1
    }
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
- Physics handles are optional until the corresponding subsystem is active. A zero handle or `constraint_index = -1` means that no event was observed in that frame.
- `physics.body` contains only scalar state and pointers captured at the static body boundaries. `mass` maps to body offset `+0x4C`, `inverse_mass` to `+0x5C`, and `state_index` to `+0x60`.
- A body record is correlated by `body_ptr` across the shape batch, aggregate preparation, dynamic update, and normalization hooks. Do not infer solver completion from entering the registration functions alone.

## Mono and stereo comparison

Capture the same stationary scene in `mono`, `left`, and `right` phases. The camera owner and gameplay state should remain stable. The eye packet, projection constants, culling range, plane resource, and completion token are compared by `frame_id`. A stereo frame is valid only when both eyes have distinct view or image handles and both reach completion before release.

Keep these records beside the private runtime trace. Commit derived aggregates or field names after review, not the raw capture.
