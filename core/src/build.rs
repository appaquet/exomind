use exocore_protos::{
    core::BuildInfo,
    prost::{ProstDateTimeExt, ProstTimestampExt},
};
use shadow_rs::shadow;

shadow!(build);

pub fn build_info() -> BuildInfo {
    let build_time = chrono::DateTime::parse_from_rfc3339(build::BUILD_TIME_3339)
        .expect("Couldn't parse build time");

    BuildInfo {
        version: build::PKG_VERSION.to_string(),
        build_time: Some(build_time.to_proto_timestamp()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_info_smoke_test() {
        let info = build_info_str();
        assert!(!info.is_empty());
    }
}
