use crate::abi::{
    c_str_to_string, JamSeqAudioBuffer, JamSeqNoteName, JamSeqParamType, JamSeqParameter,
    JamSeqPluginDescriptor, JamSeqPluginType,
};

/// High-level, owned plugin descriptor suitable for IPC and UI display.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PluginDescriptor {
    pub name: String,
    pub uri_or_path: String,
    pub format: String,
    pub plugin_type: JamSeqPluginType,
}

impl PluginDescriptor {
    pub fn new(
        name: impl Into<String>,
        uri_or_path: impl Into<String>,
        format: impl Into<String>,
        plugin_type: JamSeqPluginType,
    ) -> Self {
        Self {
            name: name.into(),
            uri_or_path: uri_or_path.into(),
            format: format.into(),
            plugin_type,
        }
    }

    /// Construct from raw C-ABI descriptor.
    ///
    /// # Safety
    /// String pointers in `raw` must be valid, null-terminated C strings.
    pub unsafe fn from_raw(raw: &JamSeqPluginDescriptor) -> Self {
        unsafe {
            Self {
                name: c_str_to_string(raw.name),
                uri_or_path: c_str_to_string(raw.uri_or_path),
                format: c_str_to_string(raw.format),
                plugin_type: raw.plugin_type,
            }
        }
    }
}

/// High-level, owned parameter descriptor for a plugin instance.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterInfo {
    pub id: String,
    pub name: String,
    pub group: String,
    pub min: f32,
    pub max: f32,
    pub default_value: f32,
    pub current_value: f32,
    pub param_type: JamSeqParamType,
}

impl ParameterInfo {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        group: impl Into<String>,
        min: f32,
        max: f32,
        default_value: f32,
        current_value: f32,
        param_type: JamSeqParamType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            group: group.into(),
            min,
            max,
            default_value,
            current_value,
            param_type,
        }
    }

    /// Construct from raw C-ABI parameter.
    ///
    /// # Safety
    /// String pointers in `raw` must be valid, null-terminated C strings.
    pub unsafe fn from_raw(raw: &JamSeqParameter) -> Self {
        unsafe {
            Self {
                id: c_str_to_string(raw.id),
                name: c_str_to_string(raw.name),
                group: c_str_to_string(raw.group),
                min: raw.min,
                max: raw.max,
                default_value: raw.default_value,
                current_value: raw.current_value,
                param_type: raw.param_type,
            }
        }
    }
}

/// High-level, owned drum / note mapping entry.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoteNameInfo {
    pub pitch: u8,
    pub name: String,
}

impl NoteNameInfo {
    pub fn new(pitch: u8, name: impl Into<String>) -> Self {
        Self {
            pitch,
            name: name.into(),
        }
    }

    /// Construct from raw C-ABI note name struct.
    pub fn from_raw(raw: &JamSeqNoteName) -> Self {
        Self {
            pitch: raw.pitch,
            name: raw.name_to_string(),
        }
    }

    /// Convert to raw C-ABI note name struct.
    pub fn to_raw(&self) -> JamSeqNoteName {
        JamSeqNoteName::new(self.pitch, &self.name)
    }
}

/// Real-time safe, stack-allocated audio buffer view for input channels (`&[&[f32]]`).
/// Avoids any heap allocation in real-time DSP loops.
pub struct StackAudioInputView<const MAX_CHANNELS: usize = 8> {
    channel_ptrs: [*mut f32; MAX_CHANNELS],
    buffer: JamSeqAudioBuffer,
}

impl<const MAX_CHANNELS: usize> StackAudioInputView<MAX_CHANNELS> {
    pub fn new(channels: &[&[f32]]) -> Self {
        let num_channels = channels.len().min(MAX_CHANNELS);
        let num_samples = channels.first().map_or(0, |c| c.len()) as u32;

        let mut channel_ptrs = [std::ptr::null_mut(); MAX_CHANNELS];
        for (i, chan) in channels.iter().take(num_channels).enumerate() {
            // Safe in C-ABI read-only context where inputs are declared `*const JamSeqAudioBuffer`
            channel_ptrs[i] = chan.as_ptr() as *mut f32;
        }

        let buffer = JamSeqAudioBuffer {
            num_channels: num_channels as u32,
            num_samples,
            channel_data: channel_ptrs.as_mut_ptr(),
        };

        Self {
            channel_ptrs,
            buffer,
        }
    }

    pub fn as_raw(&mut self) -> &JamSeqAudioBuffer {
        // Update pointer in case struct was moved
        self.buffer.channel_data = self.channel_ptrs.as_mut_ptr();
        &self.buffer
    }
}

/// Real-time safe, stack-allocated audio buffer view for output channels (`&mut [&mut [f32]]`).
/// Avoids any heap allocation in real-time DSP loops.
pub struct StackAudioOutputView<const MAX_CHANNELS: usize = 8> {
    channel_ptrs: [*mut f32; MAX_CHANNELS],
    buffer: JamSeqAudioBuffer,
}

impl<const MAX_CHANNELS: usize> StackAudioOutputView<MAX_CHANNELS> {
    pub fn new(channels: &mut [&mut [f32]]) -> Self {
        let num_channels = channels.len().min(MAX_CHANNELS);
        let num_samples = channels.first().map_or(0, |c| c.len()) as u32;

        let mut channel_ptrs = [std::ptr::null_mut(); MAX_CHANNELS];
        for (i, chan) in channels.iter_mut().take(num_channels).enumerate() {
            channel_ptrs[i] = chan.as_mut_ptr();
        }

        let buffer = JamSeqAudioBuffer {
            num_channels: num_channels as u32,
            num_samples,
            channel_data: channel_ptrs.as_mut_ptr(),
        };

        Self {
            channel_ptrs,
            buffer,
        }
    }

    pub fn as_raw_mut(&mut self) -> &mut JamSeqAudioBuffer {
        // Update pointer in case struct was moved
        self.buffer.channel_data = self.channel_ptrs.as_mut_ptr();
        &mut self.buffer
    }
}
