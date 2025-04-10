use ndi_sdk_sys::sdk;

fn main() {
    let v = sdk::version();

    println!("{}", v.unwrap_or("NDI SDK version unavailable"));

    sdk::initialize().unwrap();

    println!("NDI initialized successfully");

    sdk::destroy();

    println!("NDI destroyed successfully");
}
