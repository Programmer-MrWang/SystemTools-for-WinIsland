// SystemTools —— WinIsland ABI v2 插件基础框架。
// 当前不声明任何能力（capabilities: 0），仅提供可安装、可加载、可卸载的骨架。
// 之后要扩展时：在 capabilities 里按位加上需要的 CAP_*，并在 create 里通过 Host 获取对应服务。
// version / author / description 由 Cargo.toml 注入，与 plugin.yml 保持一致即可。

use std::ffi::c_void;

use winisland_plugin_api::abi::{
    ABI_VERSION_2, PluginCreateInfoV2, PluginDescriptorV2, PluginHandleV2, PluginStatus,
};
use winisland_plugin_api::{PluginMetadataC, PluginToken};

struct Instance;

static DESCRIPTOR: PluginDescriptorV2 = PluginDescriptorV2 {
    struct_size: std::mem::size_of::<PluginDescriptorV2>() as u32,
    abi_version: ABI_VERSION_2,
    capabilities: 0,
    metadata: PluginMetadataC::new(
        "systemtools",
        "SystemTools",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_DESCRIPTION"),
    ),
    create: Some(create),
    shutdown: Some(shutdown),
    destroy: Some(destroy),
    on_tick: None,
};

unsafe extern "C" fn create(
    info: *const PluginCreateInfoV2,
    out_handle: *mut PluginHandleV2,
) -> PluginStatus {
    if info.is_null() || out_handle.is_null() {
        return PluginStatus::InvalidArgument;
    }
    // SAFETY: WinIsland supplies a readable create-info header.
    let info = unsafe { &*info };
    if info.struct_size < std::mem::size_of::<PluginCreateInfoV2>() as u32
        || info.abi_version != ABI_VERSION_2
        || info.plugin_token == PluginToken::INVALID
    {
        return PluginStatus::UnsupportedVersion;
    }
    // 后续扩展点：Host::from_raw(info.host_api, info.plugin_token) 之后申请服务资源，
    // 并把 RAII 句柄存进 Instance，在 shutdown 里释放。
    let instance = Box::new(Instance);
    // SAFETY: WinIsland treats this pointer as opaque until destroy.
    unsafe { out_handle.write(Box::into_raw(instance).cast::<c_void>()) };
    PluginStatus::Ok
}

unsafe extern "C" fn shutdown(_handle: PluginHandleV2) -> PluginStatus {
    PluginStatus::Ok
}

unsafe extern "C" fn destroy(handle: PluginHandleV2) {
    if !handle.is_null() {
        // SAFETY: WinIsland calls destroy once after successful shutdown.
        unsafe { drop(Box::from_raw(handle.cast::<Instance>())) };
    }
}

/// # Safety
/// WinIsland calls this exported symbol using the ABI v2 entry signature.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn winisland_plugin_entry_v2() -> *const PluginDescriptorV2 {
    &DESCRIPTOR
}
