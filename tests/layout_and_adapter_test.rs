use std::ffi::c_char;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use jamseq_plugin_api::{
    allocate_c_note_name_list, allocate_c_parameter_list, allocate_c_plugin_list,
    free_c_note_name_list, free_c_parameter_list, free_c_plugin_list, AudioPluginInstance,
    JamSeqAdapterVTable, JamSeqAudioBuffer, JamSeqMidiEvent, JamSeqNoteName, JamSeqNoteNameList,
    JamSeqParamType, JamSeqParameter, JamSeqParameterList, JamSeqPluginDescriptor,
    JamSeqPluginList, JamSeqPluginType, NoteNameInfo, ParameterInfo, PluginApiError,
    PluginDescriptor, PluginHostAdapter, StackAudioInputView, StackAudioOutputView,
    JAMSEQ_API_VERSION,
};

#[test]
fn test_c_abi_sizes_and_alignments() {
    // Audio Buffer
    assert_eq!(std::mem::size_of::<JamSeqAudioBuffer>(), 16);
    assert_eq!(std::mem::align_of::<JamSeqAudioBuffer>(), 8);
    assert_eq!(std::mem::offset_of!(JamSeqAudioBuffer, num_channels), 0);
    assert_eq!(std::mem::offset_of!(JamSeqAudioBuffer, num_samples), 4);
    assert_eq!(std::mem::offset_of!(JamSeqAudioBuffer, channel_data), 8);

    // MIDI Event
    assert_eq!(std::mem::size_of::<JamSeqMidiEvent>(), 8);
    assert_eq!(std::mem::align_of::<JamSeqMidiEvent>(), 4);
    assert_eq!(std::mem::offset_of!(JamSeqMidiEvent, tick_offset), 0);
    assert_eq!(std::mem::offset_of!(JamSeqMidiEvent, status), 4);
    assert_eq!(std::mem::offset_of!(JamSeqMidiEvent, data1), 5);
    assert_eq!(std::mem::offset_of!(JamSeqMidiEvent, data2), 6);
    assert_eq!(std::mem::offset_of!(JamSeqMidiEvent, _pad), 7);

    // Note Name
    assert_eq!(std::mem::size_of::<JamSeqNoteName>(), 65);
    assert_eq!(std::mem::align_of::<JamSeqNoteName>(), 1);
    assert_eq!(std::mem::offset_of!(JamSeqNoteName, pitch), 0);
    assert_eq!(std::mem::offset_of!(JamSeqNoteName, name), 1);

    // Note Name List
    assert_eq!(std::mem::size_of::<JamSeqNoteNameList>(), 16);
    assert_eq!(std::mem::align_of::<JamSeqNoteNameList>(), 8);
    assert_eq!(std::mem::offset_of!(JamSeqNoteNameList, count), 0);
    assert_eq!(std::mem::offset_of!(JamSeqNoteNameList, entries), 8);

    // Plugin Type Enum
    assert_eq!(std::mem::size_of::<JamSeqPluginType>(), 4);
    assert_eq!(std::mem::align_of::<JamSeqPluginType>(), 4);

    // Plugin Descriptor
    assert_eq!(std::mem::size_of::<JamSeqPluginDescriptor>(), 32);
    assert_eq!(std::mem::align_of::<JamSeqPluginDescriptor>(), 8);
    assert_eq!(std::mem::offset_of!(JamSeqPluginDescriptor, name), 0);
    assert_eq!(std::mem::offset_of!(JamSeqPluginDescriptor, uri_or_path), 8);
    assert_eq!(std::mem::offset_of!(JamSeqPluginDescriptor, format), 16);
    assert_eq!(std::mem::offset_of!(JamSeqPluginDescriptor, plugin_type), 24);

    // Plugin List
    assert_eq!(std::mem::size_of::<JamSeqPluginList>(), 16);
    assert_eq!(std::mem::align_of::<JamSeqPluginList>(), 8);
    assert_eq!(std::mem::offset_of!(JamSeqPluginList, count), 0);
    assert_eq!(std::mem::offset_of!(JamSeqPluginList, entries), 8);

    // Param Type Enum
    assert_eq!(std::mem::size_of::<JamSeqParamType>(), 4);
    assert_eq!(std::mem::align_of::<JamSeqParamType>(), 4);

    // Parameter
    assert_eq!(std::mem::size_of::<JamSeqParameter>(), 48);
    assert_eq!(std::mem::align_of::<JamSeqParameter>(), 8);
    assert_eq!(std::mem::offset_of!(JamSeqParameter, id), 0);
    assert_eq!(std::mem::offset_of!(JamSeqParameter, name), 8);
    assert_eq!(std::mem::offset_of!(JamSeqParameter, group), 16);
    assert_eq!(std::mem::offset_of!(JamSeqParameter, min), 24);
    assert_eq!(std::mem::offset_of!(JamSeqParameter, max), 28);
    assert_eq!(std::mem::offset_of!(JamSeqParameter, default_value), 32);
    assert_eq!(std::mem::offset_of!(JamSeqParameter, current_value), 36);
    assert_eq!(std::mem::offset_of!(JamSeqParameter, param_type), 40);

    // Parameter List
    assert_eq!(std::mem::size_of::<JamSeqParameterList>(), 16);
    assert_eq!(std::mem::align_of::<JamSeqParameterList>(), 8);

    // VTable
    assert_eq!(std::mem::size_of::<JamSeqAdapterVTable>(), 104);
    assert_eq!(std::mem::align_of::<JamSeqAdapterVTable>(), 8);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, api_version), 0);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, format_id), 8);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, handles), 16);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, scan), 24);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, free_plugin_list), 32);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, create_instance), 40);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, process), 48);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, get_parameters), 56);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, free_parameters), 64);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, set_parameter), 72);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, get_note_names), 80);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, free_note_names), 88);
    assert_eq!(std::mem::offset_of!(JamSeqAdapterVTable, destroy_instance), 96);
}

#[test]
fn test_note_name_construction_and_list_lifecycle() {
    let raw = JamSeqNoteName::new(36, "Bass Drum 1");
    assert_eq!(raw.pitch, 36);
    assert_eq!(raw.name_as_str().unwrap(), "Bass Drum 1");
    assert_eq!(raw.name_to_string(), "Bass Drum 1");

    let info = NoteNameInfo::from_raw(&raw);
    assert_eq!(info.pitch, 36);
    assert_eq!(info.name, "Bass Drum 1");

    let back = info.to_raw();
    assert_eq!(raw, back);

    // Test list allocation and deallocation
    let notes = vec![
        JamSeqNoteName::new(36, "Kick"),
        JamSeqNoteName::new(38, "Snare"),
        JamSeqNoteName::new(42, "Closed Hat"),
    ];
    let list_ptr = allocate_c_note_name_list(&notes);
    assert!(!list_ptr.is_null());
    unsafe {
        let list = &*list_ptr;
        assert_eq!(list.count, 3);
        let slice = std::slice::from_raw_parts(list.entries, list.count as usize);
        assert_eq!(slice[0].name_as_str().unwrap(), "Kick");
        assert_eq!(slice[1].name_as_str().unwrap(), "Snare");
        assert_eq!(slice[2].name_as_str().unwrap(), "Closed Hat");
        free_c_note_name_list(list_ptr);
    }
}

#[test]
fn test_plugin_list_and_parameter_list_lifecycle() {
    let plugins = vec![
        (
            "Mock Synth".to_string(),
            "mock:synth".to_string(),
            "mock".to_string(),
            JamSeqPluginType::Instrument,
        ),
        (
            "Mock Delay".to_string(),
            "mock:delay".to_string(),
            "mock".to_string(),
            JamSeqPluginType::AudioEffect,
        ),
    ];

    let p_list = allocate_c_plugin_list(&plugins);
    assert!(!p_list.is_null());
    unsafe {
        let list = &*p_list;
        assert_eq!(list.count, 2);
        let slice = std::slice::from_raw_parts(list.entries, list.count as usize);
        let p0 = PluginDescriptor::from_raw(&slice[0]);
        let p1 = PluginDescriptor::from_raw(&slice[1]);
        assert_eq!(p0.name, "Mock Synth");
        assert_eq!(p0.plugin_type, JamSeqPluginType::Instrument);
        assert_eq!(p1.name, "Mock Delay");
        assert_eq!(p1.plugin_type, JamSeqPluginType::AudioEffect);
        free_c_plugin_list(p_list);
    }

    let params = vec![(
        "cutoff".to_string(),
        "Filter Cutoff".to_string(),
        "Filter".to_string(),
        20.0,
        20000.0,
        1000.0,
        500.0,
        JamSeqParamType::Float,
    )];
    let param_list = allocate_c_parameter_list(&params);
    assert!(!param_list.is_null());
    unsafe {
        let list = &*param_list;
        assert_eq!(list.count, 1);
        let slice = std::slice::from_raw_parts(list.entries, list.count as usize);
        let info = ParameterInfo::from_raw(&slice[0]);
        assert_eq!(info.id, "cutoff");
        assert_eq!(info.name, "Filter Cutoff");
        assert_eq!(info.current_value, 500.0);
        free_c_parameter_list(param_list);
    }
}

#[test]
fn test_zero_allocation_stack_audio_views() {
    let in_l = vec![1.0f32; 64];
    let in_r = vec![2.0f32; 64];
    let in_chans: [&[f32]; 2] = [&in_l, &in_r];

    let mut in_view = StackAudioInputView::<2>::new(&in_chans);
    let raw_in = in_view.as_raw();
    assert_eq!(raw_in.num_channels, 2);
    assert_eq!(raw_in.num_samples, 64);
    unsafe {
        let chan0 = *raw_in.channel_data.add(0);
        let chan1 = *raw_in.channel_data.add(1);
        assert_eq!(*chan0.add(0), 1.0);
        assert_eq!(*chan1.add(0), 2.0);
    }

    let mut out_l = vec![0.0f32; 64];
    let mut out_r = vec![0.0f32; 64];
    let mut out_chans: [&mut [f32]; 2] = [&mut out_l, &mut out_r];

    let mut out_view = StackAudioOutputView::<2>::new(&mut out_chans);
    let raw_out = out_view.as_raw_mut();
    assert_eq!(raw_out.num_channels, 2);
    assert_eq!(raw_out.num_samples, 64);
    unsafe {
        let chan0 = *raw_out.channel_data.add(0);
        *chan0.add(5) = 42.0;
    }
    assert_eq!(out_l[5], 42.0);
}

// ---------------------------------------------------------------------------
// Mock Adapter & Instance Tests
// ---------------------------------------------------------------------------

struct MockPluginState {
    cutoff: f32,
    destroyed: Arc<AtomicBool>,
    notes_played: Arc<AtomicU32>,
}

static MOCK_DESTROYED: AtomicBool = AtomicBool::new(false);
static MOCK_NOTES_PLAYED: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" fn mock_format_id() -> *const c_char {
    c"mock".as_ptr()
}

unsafe extern "C" fn mock_handles(path_or_uri: *const c_char) -> bool {
    if path_or_uri.is_null() {
        return false;
    }
    let s = unsafe { std::ffi::CStr::from_ptr(path_or_uri).to_str().unwrap_or("") };
    s.starts_with("mock:")
}

unsafe extern "C" fn mock_scan() -> *mut JamSeqPluginList {
    let items = vec![(
        "Mock Instrument".to_string(),
        "mock:instrument".to_string(),
        "mock".to_string(),
        JamSeqPluginType::Instrument,
    )];
    allocate_c_plugin_list(&items)
}

unsafe extern "C" fn mock_free_plugin_list(list: *mut JamSeqPluginList) {
    unsafe { free_c_plugin_list(list) };
}

unsafe extern "C" fn mock_create_instance(
    path_or_uri: *const c_char,
    _sample_rate: f64,
    _max_block_size: u32,
) -> *mut std::ffi::c_void {
    if path_or_uri.is_null() {
        return std::ptr::null_mut();
    }
    let state = Box::new(MockPluginState {
        cutoff: 440.0,
        destroyed: Arc::new(AtomicBool::new(false)),
        notes_played: Arc::new(AtomicU32::new(0)),
    });
    Box::into_raw(state) as *mut std::ffi::c_void
}

unsafe extern "C" fn mock_process(
    instance: *mut std::ffi::c_void,
    _inputs: *const JamSeqAudioBuffer,
    outputs: *mut JamSeqAudioBuffer,
    midi_events: *const JamSeqMidiEvent,
    midi_count: u32,
) {
    if instance.is_null() || outputs.is_null() {
        return;
    }
    let state = unsafe { &mut *(instance as *mut MockPluginState) };
    if midi_count > 0 && !midi_events.is_null() {
        let events = unsafe { std::slice::from_raw_parts(midi_events, midi_count as usize) };
        for ev in events {
            if ev.status & 0xF0 == 0x90 {
                state.notes_played.fetch_add(1, Ordering::SeqCst);
                MOCK_NOTES_PLAYED.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    unsafe {
        let out_buf = &*outputs;
        for c in 0..out_buf.num_channels as usize {
            let chan_ptr = *out_buf.channel_data.add(c);
            for s in 0..out_buf.num_samples as usize {
                *chan_ptr.add(s) = 0.5f32;
            }
        }
    }
}

unsafe extern "C" fn mock_get_parameters(instance: *mut std::ffi::c_void) -> *mut JamSeqParameterList {
    if instance.is_null() {
        return std::ptr::null_mut();
    }
    let state = unsafe { &*(instance as *const MockPluginState) };
    let params = vec![(
        "cutoff".to_string(),
        "Cutoff Freq".to_string(),
        "Filter".to_string(),
        20.0,
        20000.0,
        440.0,
        state.cutoff,
        JamSeqParamType::Float,
    )];
    allocate_c_parameter_list(&params)
}

unsafe extern "C" fn mock_free_parameters(list: *mut JamSeqParameterList) {
    unsafe { free_c_parameter_list(list) };
}

unsafe extern "C" fn mock_set_parameter(
    instance: *mut std::ffi::c_void,
    param_id: *const c_char,
    value: f32,
) {
    if instance.is_null() || param_id.is_null() {
        return;
    }
    let state = unsafe { &mut *(instance as *mut MockPluginState) };
    let id_str = unsafe { std::ffi::CStr::from_ptr(param_id).to_str().unwrap_or("") };
    if id_str == "cutoff" {
        state.cutoff = value;
    }
}

unsafe extern "C" fn mock_get_note_names(instance: *mut std::ffi::c_void) -> *mut JamSeqNoteNameList {
    if instance.is_null() {
        return std::ptr::null_mut();
    }
    let notes = vec![
        JamSeqNoteName::new(36, "Acoustic Kick"),
        JamSeqNoteName::new(38, "Acoustic Snare"),
    ];
    allocate_c_note_name_list(&notes)
}

unsafe extern "C" fn mock_free_note_names(list: *mut JamSeqNoteNameList) {
    unsafe { free_c_note_name_list(list) };
}

unsafe extern "C" fn mock_destroy_instance(instance: *mut std::ffi::c_void) {
    if instance.is_null() {
        return;
    }
    let state = unsafe { Box::from_raw(instance as *mut MockPluginState) };
    state.destroyed.store(true, Ordering::SeqCst);
    MOCK_DESTROYED.store(true, Ordering::SeqCst);
}

static MOCK_VTABLE: JamSeqAdapterVTable = JamSeqAdapterVTable {
    api_version: JAMSEQ_API_VERSION,
    format_id: mock_format_id,
    handles: mock_handles,
    scan: mock_scan,
    free_plugin_list: mock_free_plugin_list,
    create_instance: mock_create_instance,
    process: mock_process,
    get_parameters: mock_get_parameters,
    free_parameters: mock_free_parameters,
    set_parameter: mock_set_parameter,
    get_note_names: mock_get_note_names,
    free_note_names: mock_free_note_names,
    destroy_instance: mock_destroy_instance,
};

#[test]
fn test_plugin_host_adapter_and_instance_workflow() {
    MOCK_DESTROYED.store(false, Ordering::SeqCst);
    MOCK_NOTES_PLAYED.store(0, Ordering::SeqCst);

    let adapter = PluginHostAdapter::new(&MOCK_VTABLE).expect("Valid API version");
    assert_eq!(adapter.format_id(), "mock");
    assert!(adapter.handles("mock:instrument"));
    assert!(!adapter.handles("vst3:synth"));

    let scanned = adapter.scan();
    assert_eq!(scanned.len(), 1);
    assert_eq!(scanned[0].name, "Mock Instrument");
    assert_eq!(scanned[0].plugin_type, JamSeqPluginType::Instrument);

    // Create and exercise instance inside a block to test RAII Drop
    {
        let mut instance = adapter
            .create_instance("mock:instrument", 48000.0, 512)
            .expect("Failed to create instance");

        // Introspect note names
        let note_names = instance.get_note_names();
        assert_eq!(note_names.len(), 2);
        assert_eq!(note_names[0].pitch, 36);
        assert_eq!(note_names[0].name, "Acoustic Kick");
        assert_eq!(note_names[1].pitch, 38);
        assert_eq!(note_names[1].name, "Acoustic Snare");

        // Parameters
        let params = instance.get_parameters();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].id, "cutoff");
        assert_eq!(params[0].current_value, 440.0);

        instance.set_parameter("cutoff", 880.0);
        let updated_params = instance.get_parameters();
        assert_eq!(updated_params[0].current_value, 880.0);

        // Process audio and MIDI
        let mut out_l = vec![0.0f32; 128];
        let mut out_r = vec![0.0f32; 128];
        let mut out_chans: [&mut [f32]; 2] = [&mut out_l, &mut out_r];
        let mut out_view = StackAudioOutputView::<2>::new(&mut out_chans);

        let midi_events = vec![JamSeqMidiEvent::note_on(0, 0, 36, 100)];
        instance.process(None, out_view.as_raw_mut(), &midi_events);

        assert_eq!(out_l[0], 0.5);
        assert_eq!(out_r[0], 0.5);
        assert_eq!(MOCK_NOTES_PLAYED.load(Ordering::SeqCst), 1);

        assert!(!MOCK_DESTROYED.load(Ordering::SeqCst));
    }

    // Now instance must have been dropped and destroyed via vtable
    assert!(MOCK_DESTROYED.load(Ordering::SeqCst));
}

#[test]
fn test_api_version_mismatch_rejection() {
    static OBSOLETE_VTABLE: JamSeqAdapterVTable = JamSeqAdapterVTable {
        api_version: 9999, // Mismatched version
        format_id: mock_format_id,
        handles: mock_handles,
        scan: mock_scan,
        free_plugin_list: mock_free_plugin_list,
        create_instance: mock_create_instance,
        process: mock_process,
        get_parameters: mock_get_parameters,
        free_parameters: mock_free_parameters,
        set_parameter: mock_set_parameter,
        get_note_names: mock_get_note_names,
        free_note_names: mock_free_note_names,
        destroy_instance: mock_destroy_instance,
    };

    match PluginHostAdapter::new(&OBSOLETE_VTABLE) {
        Err(PluginApiError::ApiVersionMismatch { expected, actual }) => {
            assert_eq!(expected, JAMSEQ_API_VERSION);
            assert_eq!(actual, 9999);
        }
        _ => panic!("Expected ApiVersionMismatch error"),
    }
}
