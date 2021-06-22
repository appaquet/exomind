use exocore_protos::{
    core::BuildInfo,
    prost::{ProstDateTimeExt, ProstTimestampExt},
};
use shadow_rs::shadow;

shadow!(build);

pub fn build_info() -> BuildInfo {
    let naive_build_time =
        chrono::NaiveDateTime::parse_from_str(build::BUILD_TIME, "%Y-%m-%d %H:%M:%S")
            .expect("Couldn't parse build time");
    let utc_build_time = chrono::DateTime::<chrono::Utc>::from_utc(naive_build_time, chrono::Utc);

    BuildInfo {
        version: build::PKG_VERSION.to_string(),
        build_time: Some(utc_build_time.to_proto_timestamp()),
        debug: shadow_rs::is_debug(),
        rust_version: build::RUST_VERSION.to_string(),
    }
}

pub fn build_info_str() -> String {
    let info = build_info();
    let build_time = info.build_time.unwrap().to_chrono_datetime();

    format!(
        "version={} build_time={} debug={} rust={}",
        info.version,
        build_time.to_rfc3339(),
        info.debug,
        info.rust_version,
    )
}
