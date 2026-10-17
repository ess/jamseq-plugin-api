use std::ffi::{c_char, CStr, CString};
use std::os::raw::c_char as raw_c_char;

/// Canonical ABI version number for compatibility checking.
pub const JAMSEQ_API_VERSION: u32 = 1;

/// Canonical symbol name exported by dynamic plugin adapters.
pub const JAMSEQ_ENTRYPOINT_NAME: &str = "jamseq_get_host_adapter";

/// Null-terminated byte literal of the adapter entrypoint symbol.
pub const JAMSEQ_ENTRYPOINT_SYMBOL: &[u8] = b"jamseq_get_host_adapter\0";

/// Type signature of the exported entrypoint function in adapter dynamic libraries.
pub type JamSeqGetHostAdapterFn = unsafe extern "C" fn() -> *const JamSeqAdapterVTable;

/// High-level classification of plugin role and intended slot placement.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JamSeqPluginType {
    /// Sound generator / synthesizer / drum machine / sampler (Slot 0).
    Instrument = 0,
    /// In-line audio signal processor / effect (Inserts 1..N, Master, Aux).
    AudioEffect = 1,
    /// Passive audio analyzer / visualizer / utility / metering tap.
    Analyzer = 2,
}

/// Metadata descriptor for an available plugin format component.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JamSeqPluginDescriptor {
    /// Human-readable plugin name (null-terminated UTF-8).
    pub name: *const c_char,
    /// Format-specific identifier, path, or URI (null-terminated UTF-8).
    pub uri_or_path: *const c_char,
    /// Format identifier string: "clap", "vst3", "lv2" (null-terminated ASCII).
    pub format: *const c_char,
    /// Functional category (Instrument, AudioEffect, Analyzer).
    pub plugin_type: JamSeqPluginType,
}

/// List of discovered plugin descriptors returned by adapter scanning.
#[repr(C)]
#[derive(Debug)]
pub struct JamSeqPluginList {
    pub count: u32,
    pub entries: *mut JamSeqPluginDescriptor,
}

/// Real-time multi-channel 32-bit floating point audio buffer.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct JamSeqAudioBuffer {
    /// Number of audio channels (e.g. 1 for mono, 2 for stereo).
    pub num_channels: u32,
    /// Number of audio sample frames per channel.
    pub num_samples: u32,
    /// Array of pointers to channel sample buffers: `*channel_data[channel_idx]`.
    pub channel_data: *mut *mut f32,
}

impl Default for JamSeqAudioBuffer {
    fn default() -> Self {
        Self {
            num_channels: 0,
            num_samples: 0,
            channel_data: std::ptr::null_mut(),
        }
    }
}

/// Real-time sample-accurate MIDI event.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JamSeqMidiEvent {
    /// Frame/sample offset within the current processing block (0..num_samples - 1).
    pub tick_offset: u32,
    /// MIDI status byte (e.g. 0x90 = Note On, 0x80 = Note Off, 0xB0 = CC).
    pub status: u8,
    /// MIDI data byte 1 (e.g. note number or CC number).
    pub data1: u8,
    /// MIDI data byte 2 (e.g. velocity or CC value).
    pub data2: u8,
    /// Explicit padding byte to ensure 8-byte layout and clean alignment.
    pub _pad: u8,
}

impl JamSeqMidiEvent {
    pub const fn new(tick_offset: u32, status: u8, data1: u8, data2: u8) -> Self {
        Self {
            tick_offset,
            status,
            data1,
            data2,
            _pad: 0,
        }
    }

    pub const fn note_on(tick_offset: u32, channel: u8, note: u8, velocity: u8) -> Self {
        Self::new(tick_offset, 0x90 | (channel & 0x0F), note, velocity)
    }

    pub const fn note_off(tick_offset: u32, channel: u8, note: u8) -> Self {
        Self::new(tick_offset, 0x80 | (channel & 0x0F), note, 0)
    }
}

/// Mapping of a MIDI note pitch (0-127) to an introspected human-readable name.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JamSeqNoteName {
    /// MIDI note pitch number (0 to 127).
    pub pitch: u8,
    /// Null-terminated UTF-8 drum / key label (up to 63 bytes + null terminator).
    pub name: [raw_c_char; 64],
}

impl std::fmt::Debug for JamSeqNoteName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JamSeqNoteName")
            .field("pitch", &self.pitch)
            .field("name", &self.name_as_str().unwrap_or("<invalid utf8>"))
            .finish()
    }
}

impl PartialEq for JamSeqNoteName {
    fn eq(&self, other: &Self) -> bool {
        self.pitch == other.pitch && self.name_as_bytes() == other.name_as_bytes()
    }
}

impl Eq for JamSeqNoteName {}

impl JamSeqNoteName {
    /// Create a new `JamSeqNoteName` from pitch and string.
    pub fn new(pitch: u8, name_str: &str) -> Self {
        let mut name = [0 as raw_c_char; 64];
        let bytes = name_str.as_bytes();
        let copy_len = bytes.len().min(63);
        for (i, &b) in bytes.iter().take(copy_len).enumerate() {
            name[i] = b as raw_c_char;
        }
        name[copy_len] = 0;
        Self { pitch, name }
    }

    /// Returns the name bytes up to the first null byte.
    pub fn name_as_bytes(&self) -> &[u8] {
        let len = self
            .name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(self.name.len());
        // Cast &[c_char] to &[u8] safely.
        unsafe { std::slice::from_raw_parts(self.name.as_ptr() as *const u8, len) }
    }

    /// Read the name as a string slice.
    pub fn name_as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.name_as_bytes())
    }

    /// Read the name as an owned String.
    pub fn name_to_string(&self) -> String {
        self.name_as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|_| String::from_utf8_lossy(self.name_as_bytes()).into_owned())
    }
}

/// List of note name mappings returned by an instrument instance.
#[repr(C)]
#[derive(Debug)]
pub struct JamSeqNoteNameList {
    pub count: u32,
    pub entries: *mut JamSeqNoteName,
}

/// Parameter value data type.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JamSeqParamType {
    Float = 0,
    Bool = 1,
    Int = 2,
    Enum = 3,
}

/// Parameter definition for a plugin instance.
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct JamSeqParameter {
    /// Unique identifier string (null-terminated UTF-8).
    pub id: *const c_char,
    /// Human-readable display label (null-terminated UTF-8).
    pub name: *const c_char,
    /// Hierarchical group or module path (e.g. "Filter/Cutoff", null-terminated UTF-8).
    pub group: *const c_char,
    /// Minimum allowed float value.
    pub min: f32,
    /// Maximum allowed float value.
    pub max: f32,
    /// Default float value.
    pub default_value: f32,
    /// Current float value.
    pub current_value: f32,
    /// Parameter data type.
    pub param_type: JamSeqParamType,
}

/// List of exposed parameters returned by an instance.
#[repr(C)]
#[derive(Debug)]
pub struct JamSeqParameterList {
    pub count: u32,
    pub entries: *mut JamSeqParameter,
}

/// Canonical C-ABI function pointer vtable implemented by dynamic plugin format adapters.
#[repr(C)]
pub struct JamSeqAdapterVTable {
    /// ABI version identifier for safety verification (must equal `JAMSEQ_API_VERSION`).
    pub api_version: u32,

    /// Returns a static null-terminated ASCII string identifying the format ("clap", "vst3", "lv2").
    pub format_id: unsafe extern "C" fn() -> *const c_char,

    /// Returns true if this adapter can handle the given path or URI string.
    pub handles: unsafe extern "C" fn(path_or_uri: *const c_char) -> bool,

    /// Discovers installed plugins for this format on the current machine.
    /// Caller takes ownership of the returned list and must free it via `free_plugin_list`.
    pub scan: unsafe extern "C" fn() -> *mut JamSeqPluginList,

    /// Deallocates a `JamSeqPluginList` allocated by this adapter's `scan`.
    pub free_plugin_list: unsafe extern "C" fn(list: *mut JamSeqPluginList),

    /// Instantiates a plugin instance. Returns an opaque pointer to the instance handle, or null on error.
    pub create_instance: unsafe extern "C" fn(
        path_or_uri: *const c_char,
        sample_rate: f64,
        max_block_size: u32,
    ) -> *mut std::ffi::c_void,

    /// Real-time sample processing block.
    /// MUST be non-blocking and real-time safe (no allocations, locks, or system calls).
    pub process: unsafe extern "C" fn(
        instance: *mut std::ffi::c_void,
        inputs: *const JamSeqAudioBuffer,
        outputs: *mut JamSeqAudioBuffer,
        midi_events: *const JamSeqMidiEvent,
        midi_count: u32,
    ),

    /// Introspects exposed parameters for the instance.
    /// Caller takes ownership of the list and must free it via `free_parameters`.
    pub get_parameters: unsafe extern "C" fn(instance: *mut std::ffi::c_void) -> *mut JamSeqParameterList,

    /// Deallocates a `JamSeqParameterList` allocated by this adapter's `get_parameters`.
    pub free_parameters: unsafe extern "C" fn(list: *mut JamSeqParameterList),

    /// Updates parameter value for a given parameter identifier.
    pub set_parameter: unsafe extern "C" fn(
        instance: *mut std::ffi::c_void,
        param_id: *const c_char,
        value: f32,
    ),

    /// Introspects mapped drum and note names (CLAP note-name, LV2 midnam, VST3 IUnitInfo).
    /// Returns null or an empty list if unsupported.
    /// Caller takes ownership and must free it via `free_note_names`.
    pub get_note_names: unsafe extern "C" fn(instance: *mut std::ffi::c_void) -> *mut JamSeqNoteNameList,

    /// Deallocates a `JamSeqNoteNameList` allocated by this adapter's `get_note_names`.
    pub free_note_names: unsafe extern "C" fn(list: *mut JamSeqNoteNameList),

    /// Destroys the plugin instance and frees all associated resources.
    pub destroy_instance: unsafe extern "C" fn(instance: *mut std::ffi::c_void),
}

// ---------------------------------------------------------------------------
// Adapter Memory Allocation & Deallocation Helpers
// ---------------------------------------------------------------------------

/// Helper for adapters to allocate a `JamSeqNoteNameList` on the heap.
pub fn allocate_c_note_name_list(notes: &[JamSeqNoteName]) -> *mut JamSeqNoteNameList {
    if notes.is_empty() {
        let boxed = Box::new(JamSeqNoteNameList {
            count: 0,
            entries: std::ptr::null_mut(),
        });
        return Box::into_raw(boxed);
    }
    let mut entries_vec = notes.to_vec();
    let entries_ptr = entries_vec.as_mut_ptr();
    std::mem::forget(entries_vec);

    let boxed = Box::new(JamSeqNoteNameList {
        count: notes.len() as u32,
        entries: entries_ptr,
    });
    Box::into_raw(boxed)
}

/// Helper for adapters to free a `JamSeqNoteNameList` allocated by `allocate_c_note_name_list`.
///
/// # Safety
/// Must only be called with a pointer allocated via `allocate_c_note_name_list`.
pub unsafe fn free_c_note_name_list(list: *mut JamSeqNoteNameList) {
    if list.is_null() {
        return;
    }
    unsafe {
        let boxed = Box::from_raw(list);
        if !boxed.entries.is_null() && boxed.count > 0 {
            let _ = Vec::from_raw_parts(boxed.entries, boxed.count as usize, boxed.count as usize);
        }
    }
}

/// Internal struct tracking string allocations for `JamSeqPluginList`.
struct AllocatedPluginListState {
    _strings: Vec<CString>,
}

/// Helper for adapters to allocate a `JamSeqPluginList` with owned C strings.
pub fn allocate_c_plugin_list(
    descriptors: &[(String, String, String, JamSeqPluginType)],
) -> *mut JamSeqPluginList {
    if descriptors.is_empty() {
        let boxed = Box::new(JamSeqPluginList {
            count: 0,
            entries: std::ptr::null_mut(),
        });
        return Box::into_raw(boxed);
    }

    let mut string_storage = Vec::with_capacity(descriptors.len() * 3);
    let mut entries = Vec::with_capacity(descriptors.len());

    for (name, uri_or_path, format, plugin_type) in descriptors {
        let c_name = CString::new(name.as_str()).unwrap_or_default();
        let c_uri = CString::new(uri_or_path.as_str()).unwrap_or_default();
        let c_fmt = CString::new(format.as_str()).unwrap_or_default();

        let name_ptr = c_name.as_ptr();
        let uri_ptr = c_uri.as_ptr();
        let fmt_ptr = c_fmt.as_ptr();

        string_storage.push(c_name);
        string_storage.push(c_uri);
        string_storage.push(c_fmt);

        entries.push(JamSeqPluginDescriptor {
            name: name_ptr,
            uri_or_path: uri_ptr,
            format: fmt_ptr,
            plugin_type: *plugin_type,
        });
    }

    let entries_ptr = entries.as_mut_ptr();
    let count = entries.len() as u32;
    std::mem::forget(entries);

    let boxed = Box::new((
        JamSeqPluginList {
            count,
            entries: entries_ptr,
        },
        AllocatedPluginListState {
            _strings: string_storage,
        },
    ));
    let raw = Box::into_raw(boxed);
    raw as *mut JamSeqPluginList
}

/// Helper for adapters to free a `JamSeqPluginList` allocated by `allocate_c_plugin_list`.
///
/// # Safety
/// Must only be called on pointers created by `allocate_c_plugin_list`.
pub unsafe fn free_c_plugin_list(list: *mut JamSeqPluginList) {
    if list.is_null() {
        return;
    }
    unsafe {
        let tuple_ptr = list as *mut (JamSeqPluginList, AllocatedPluginListState);
        let boxed = Box::from_raw(tuple_ptr);
        if !boxed.0.entries.is_null() && boxed.0.count > 0 {
            let _ = Vec::from_raw_parts(
                boxed.0.entries,
                boxed.0.count as usize,
                boxed.0.count as usize,
            );
        }
    }
}

/// Helper for adapters to allocate a `JamSeqParameterList` with owned C strings.
pub fn allocate_c_parameter_list(
    params: &[(String, String, String, f32, f32, f32, f32, JamSeqParamType)],
) -> *mut JamSeqParameterList {
    if params.is_empty() {
        let boxed = Box::new(JamSeqParameterList {
            count: 0,
            entries: std::ptr::null_mut(),
        });
        return Box::into_raw(boxed);
    }

    let mut string_storage = Vec::with_capacity(params.len() * 3);
    let mut entries = Vec::with_capacity(params.len());

    for (id, name, group, min, max, default_val, cur_val, ptype) in params {
        let c_id = CString::new(id.as_str()).unwrap_or_default();
        let c_name = CString::new(name.as_str()).unwrap_or_default();
        let c_group = CString::new(group.as_str()).unwrap_or_default();

        let id_ptr = c_id.as_ptr();
        let name_ptr = c_name.as_ptr();
        let group_ptr = c_group.as_ptr();

        string_storage.push(c_id);
        string_storage.push(c_name);
        string_storage.push(c_group);

        entries.push(JamSeqParameter {
            id: id_ptr,
            name: name_ptr,
            group: group_ptr,
            min: *min,
            max: *max,
            default_value: *default_val,
            current_value: *cur_val,
            param_type: *ptype,
        });
    }

    let entries_ptr = entries.as_mut_ptr();
    let count = entries.len() as u32;
    std::mem::forget(entries);

    let boxed = Box::new((
        JamSeqParameterList {
            count,
            entries: entries_ptr,
        },
        string_storage,
    ));
    let raw = Box::into_raw(boxed);
    raw as *mut JamSeqParameterList
}

/// Helper for adapters to free a `JamSeqParameterList` allocated by `allocate_c_parameter_list`.
///
/// # Safety
/// Must only be called on pointers created by `allocate_c_parameter_list`.
pub unsafe fn free_c_parameter_list(list: *mut JamSeqParameterList) {
    if list.is_null() {
        return;
    }
    unsafe {
        let tuple_ptr = list as *mut (JamSeqParameterList, Vec<CString>);
        let boxed = Box::from_raw(tuple_ptr);
        if !boxed.0.entries.is_null() && boxed.0.count > 0 {
            let _ = Vec::from_raw_parts(
                boxed.0.entries,
                boxed.0.count as usize,
                boxed.0.count as usize,
            );
        }
    }
}

/// Convert a C-string pointer to a safe Rust String.
///
/// # Safety
/// `ptr` must be a valid, null-terminated C string or null.
pub unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
    }
}
