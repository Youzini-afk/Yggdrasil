use yg_kernel_sdk::{AppendEventRequest, AssetGetParams, KernelClient, KERNEL_V1_ASSET_PUT};

fn generated_method_is_available(client: &KernelClient, params: AssetGetParams) {
    let _future = client.asset_get(params);
}

#[test]
fn generated_modules_are_exported_from_the_crate_root() {
    assert_eq!(KERNEL_V1_ASSET_PUT, "kernel/v1/asset.put");
    let _ = std::mem::size_of::<AppendEventRequest>();
    let _ = generated_method_is_available;
}
