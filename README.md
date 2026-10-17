# `jamseq-plugin-api`

**C-ABI contract and safe Rust abstractions for pluggable audio plugin adapters in the Jamband ecosystem.**

---

## 1. Overview

`jamseq-plugin-api` defines the low-level, binary-stable C interface (`#[repr(C)]`) and high-level safe Rust traits that decouple the core `jamseq` audio sequencer from format-specific plugin hosting libraries (such as CLAP, VST3, and LV2).

### Architectural Philosophy
1. **Zero Monolithic Dependencies**: Core `jamseq` contains no Lilv, Clack, or VST3 COM headers.
2. **Binary ABI Stability**: Rust does not provide a stable across-compiler ABI. The C-ABI vtable contract (`JamSeqAdapterVTable`) ensures that adapter libraries compiled separately link and execute safely without compiler version coupling.
3. **First-Class Drum & Note Name Introspection**: Mapped drum labels (CLAP `clap.note-name`, LV2 `midnam`, VST3 `IUnitInfo`) are included in the C-ABI from day one.
4. **Zero Heap Allocation in Real-Time Path**: Stack-allocated buffer views (`StackAudioInputView`, `StackAudioOutputView`) allow `process` blocks to execute without audio thread allocations.

---

## 2. Core C-ABI Structures

- **`JamSeqAdapterVTable`**: The canonical function pointer table exported by adapter dynamic libraries via `jamseq_get_host_adapter`.
- **`JamSeqAudioBuffer`**: Real-time multi-channel 32-bit floating point audio buffer.
- **`JamSeqMidiEvent`**: Frame-accurate MIDI status and data tuple (8-byte layout).
- **`JamSeqNoteName` & `JamSeqNoteNameList`**: Mapped key/drum label pairs (`pitch: u8`, `name: [c_char; 64]`).
- **`JamSeqParameter` & `JamSeqParameterList`**: Introspected plugin parameter metadata.
- **`JamSeqPluginDescriptor` & `JamSeqPluginList`**: Discovered plugin descriptors and categories (`Instrument`, `AudioEffect`, `Analyzer`).

---

## 3. Safe Rust Abstractions

- **`PluginHostAdapter`**: Safe client wrapper over an adapter's `JamSeqAdapterVTable`, managing library lifetime and safe scanning/instantiation.
- **`AudioPluginInstance`**: Trait representing an active plugin instance.
- **`RawPluginInstance`**: Safe RAII wrapper ensuring `destroy_instance` is called on drop.

---

## 4. Verification

Run the test suite to verify layout and alignment stability:
```bash
cargo test -p jamseq-plugin-api
```
