use std::any::Any;
use std::ffi::{CStr, CString};
use std::sync::Arc;

use crate::abi::{JamSeqAdapterVTable, JamSeqAudioBuffer, JamSeqMidiEvent, JAMSEQ_API_VERSION};
use crate::error::PluginApiError;
use crate::types::{NoteNameInfo, ParameterInfo, PluginDescriptor};

/// Safe Rust interface representing an instantiated audio plugin (CLAP, VST3, LV2, etc.).
pub trait AudioPluginInstance: Send {
    /// Process a block of audio and MIDI events.
    ///
    /// Must be real-time safe (no heap allocations, locks, or blocking system calls).
    fn process(
        &mut self,
        inputs: Option<&JamSeqAudioBuffer>,
        outputs: &mut JamSeqAudioBuffer,
        midi_events: &[JamSeqMidiEvent],
    );

    /// Introspect exposed parameters from the plugin instance.
    fn get_parameters(&mut self) -> Vec<ParameterInfo>;

    /// Update a parameter by string ID.
    fn set_parameter(&mut self, param_id: &str, value: f32);

    /// Introspect custom note / drum mapping names (e.g. Note 36 = "Kick Drum").
    fn get_note_names(&mut self) -> Vec<NoteNameInfo>;
}

/// Concrete RAII wrapper around a raw C-ABI plugin instance pointer.
///
/// Ensures `vtable.destroy_instance` is reliably invoked when the instance is dropped.
pub struct RawPluginInstance {
    raw: *mut std::ffi::c_void,
    vtable: &'static JamSeqAdapterVTable,
    _owner: Option<Arc<dyn Any + Send + Sync>>,
}

// Raw C pointers are not Send by default, but plugin instances are designed
// to be safely transferred across threads (e.g. between loader and audio thread).
unsafe impl Send for RawPluginInstance {}

impl RawPluginInstance {
    /// Construct a new RAII wrapper over an instantiated raw handle.
    ///
    /// # Safety
    /// `raw` must be a valid instance pointer returned by `vtable.create_instance`.
    pub unsafe fn new(
        raw: *mut std::ffi::c_void,
        vtable: &'static JamSeqAdapterVTable,
        owner: Option<Arc<dyn Any + Send + Sync>>,
    ) -> Result<Self, PluginApiError> {
        if raw.is_null() {
            return Err(PluginApiError::InstantiationFailed("null handle".into()));
        }
        Ok(Self {
            raw,
            vtable,
            _owner: owner,
        })
    }

    /// Access the underlying raw pointer if needed for low-level interop.
    pub fn raw_handle(&self) -> *mut std::ffi::c_void {
        self.raw
    }

    /// Access the underlying vtable.
    pub fn vtable(&self) -> &'static JamSeqAdapterVTable {
        self.vtable
    }
}

impl AudioPluginInstance for RawPluginInstance {
    fn process(
        &mut self,
        inputs: Option<&JamSeqAudioBuffer>,
        outputs: &mut JamSeqAudioBuffer,
        midi_events: &[JamSeqMidiEvent],
    ) {
        let input_ptr = inputs.map_or(std::ptr::null(), |b| b as *const JamSeqAudioBuffer);
        let midi_ptr = if midi_events.is_empty() {
            std::ptr::null()
        } else {
            midi_events.as_ptr()
        };

        unsafe {
            (self.vtable.process)(
                self.raw,
                input_ptr,
                outputs as *mut JamSeqAudioBuffer,
                midi_ptr,
                midi_events.len() as u32,
            );
        }
    }

    fn get_parameters(&mut self) -> Vec<ParameterInfo> {
        let mut result = Vec::new();
        unsafe {
            let list_ptr = (self.vtable.get_parameters)(self.raw);
            if !list_ptr.is_null() {
                let list = &*list_ptr;
                if !list.entries.is_null() && list.count > 0 {
                    let slice = std::slice::from_raw_parts(list.entries, list.count as usize);
                    for item in slice {
                        result.push(ParameterInfo::from_raw(item));
                    }
                }
                (self.vtable.free_parameters)(list_ptr);
            }
        }
        result
    }

    fn set_parameter(&mut self, param_id: &str, value: f32) {
        if let Ok(c_id) = CString::new(param_id) {
            unsafe {
                (self.vtable.set_parameter)(self.raw, c_id.as_ptr(), value);
            }
        }
    }

    fn get_note_names(&mut self) -> Vec<NoteNameInfo> {
        let mut result = Vec::new();
        unsafe {
            let list_ptr = (self.vtable.get_note_names)(self.raw);
            if !list_ptr.is_null() {
                let list = &*list_ptr;
                if !list.entries.is_null() && list.count > 0 {
                    let slice = std::slice::from_raw_parts(list.entries, list.count as usize);
                    for item in slice {
                        result.push(NoteNameInfo::from_raw(item));
                    }
                }
                (self.vtable.free_note_names)(list_ptr);
            }
        }
        result
    }
}

impl Drop for RawPluginInstance {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                (self.vtable.destroy_instance)(self.raw);
            }
            self.raw = std::ptr::null_mut();
        }
    }
}

/// Safe client interface wrapping an adapter vtable (e.g. loaded from a `.so` / `.dll`).
#[derive(Clone)]
pub struct PluginHostAdapter {
    vtable: &'static JamSeqAdapterVTable,
    owner: Option<Arc<dyn Any + Send + Sync>>,
}

impl PluginHostAdapter {
    /// Create a new adapter interface from a static vtable reference.
    pub fn new(vtable: &'static JamSeqAdapterVTable) -> Result<Self, PluginApiError> {
        if vtable.api_version != JAMSEQ_API_VERSION {
            return Err(PluginApiError::ApiVersionMismatch {
                expected: JAMSEQ_API_VERSION,
                actual: vtable.api_version,
            });
        }
        Ok(Self {
            vtable,
            owner: None,
        })
    }

    /// Create a new adapter interface holding an owner object (such as a `libloading::Library` handle).
    pub fn with_owner(
        vtable: &'static JamSeqAdapterVTable,
        owner: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, PluginApiError> {
        if vtable.api_version != JAMSEQ_API_VERSION {
            return Err(PluginApiError::ApiVersionMismatch {
                expected: JAMSEQ_API_VERSION,
                actual: vtable.api_version,
            });
        }
        Ok(Self {
            vtable,
            owner: Some(owner),
        })
    }

    /// Return the format identifier string ("clap", "vst3", "lv2").
    pub fn format_id(&self) -> &str {
        unsafe {
            let ptr = (self.vtable.format_id)();
            if ptr.is_null() {
                "unknown"
            } else {
                CStr::from_ptr(ptr).to_str().unwrap_or("unknown")
            }
        }
    }

    /// Returns true if this adapter handles the specified plugin path or URI.
    pub fn handles(&self, path_or_uri: &str) -> bool {
        if let Ok(c_str) = CString::new(path_or_uri) {
            unsafe { (self.vtable.handles)(c_str.as_ptr()) }
        } else {
            false
        }
    }

    /// Scan system paths for plugins supported by this format adapter.
    pub fn scan(&self) -> Vec<PluginDescriptor> {
        let mut result = Vec::new();
        unsafe {
            let list_ptr = (self.vtable.scan)();
            if !list_ptr.is_null() {
                let list = &*list_ptr;
                if !list.entries.is_null() && list.count > 0 {
                    let slice = std::slice::from_raw_parts(list.entries, list.count as usize);
                    for item in slice {
                        result.push(PluginDescriptor::from_raw(item));
                    }
                }
                (self.vtable.free_plugin_list)(list_ptr);
            }
        }
        result
    }

    /// Instantiate a plugin instance.
    pub fn create_instance(
        &self,
        path_or_uri: &str,
        sample_rate: f64,
        max_block_size: u32,
    ) -> Result<RawPluginInstance, PluginApiError> {
        let c_str = CString::new(path_or_uri)?;
        let raw_instance = unsafe {
            (self.vtable.create_instance)(c_str.as_ptr(), sample_rate, max_block_size)
        };

        if raw_instance.is_null() {
            return Err(PluginApiError::InstantiationFailed(path_or_uri.to_string()));
        }

        unsafe { RawPluginInstance::new(raw_instance, self.vtable, self.owner.clone()) }
    }

    /// Access the underlying vtable directly.
    pub fn vtable(&self) -> &'static JamSeqAdapterVTable {
        self.vtable
    }
}
