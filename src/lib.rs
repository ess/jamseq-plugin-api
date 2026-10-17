//! `jamseq-plugin-api`: C-ABI and safe Rust contract for dynamic audio plugin adapters in Jamband.
//!
//! This crate provides the C-compatible vtable (`JamSeqAdapterVTable`), real-time audio/MIDI
//! buffer definitions, and safe Rust abstractions (`PluginHostAdapter`, `AudioPluginInstance`)
//! that enable `jamseq` to dynamically load format adapters (`.so` / `.dll` / `.dylib`)
//! at runtime with zero direct compile-time coupling to Lilv, Clack, or VST3.

pub mod abi;
pub mod error;
pub mod instance;
pub mod types;

// Re-export primary types at crate root for ergonomic consumption
pub use abi::{
    allocate_c_note_name_list, allocate_c_parameter_list, allocate_c_plugin_list, c_str_to_string,
    free_c_note_name_list, free_c_parameter_list, free_c_plugin_list, JamSeqAdapterVTable,
    JamSeqAudioBuffer, JamSeqGetHostAdapterFn, JamSeqMidiEvent, JamSeqNoteName, JamSeqNoteNameList,
    JamSeqParamType, JamSeqParameter, JamSeqParameterList, JamSeqPluginDescriptor,
    JamSeqPluginList, JamSeqPluginType, JAMSEQ_API_VERSION, JAMSEQ_ENTRYPOINT_NAME,
    JAMSEQ_ENTRYPOINT_SYMBOL,
};

pub use error::PluginApiError;
pub use instance::{AudioPluginInstance, PluginHostAdapter, RawPluginInstance};
pub use types::{
    NoteNameInfo, ParameterInfo, PluginDescriptor, StackAudioInputView, StackAudioOutputView,
};
