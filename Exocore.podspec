Pod::Spec.new do |spec|
    spec.name         = 'Exocore'
    spec.version      = '0.1.16'
    spec.license      = { :type => 'Apache-2.0' }
    spec.summary      = 'Distributed applications framework'
    spec.authors      = { 'Andre-Philippe Paquet' => 'appaquet@gmail.com' }
    spec.source       = { :http => 'https://github.com/appaquet/exocore/releases/download/v' + spec.version.to_s + '/exocore-ios.tar.gz', :type => 'tgz' }
    spec.homepage     = 'https://github.com/appaquet/exocore'

    spec.swift_version = '4.2'
    spec.ios.deployment_target = '11.0'

    # Rust binaries don't have bitcode (would need to build with https://github.com/getditto/rust-bitcode)
    # Exclude arm64 on iOS, since it's arm64 for iDevices, not arm64 for MacOS (see https://github.com/CocoaPods/CocoaPods/issues/10104)
    spec.pod_target_xcconfig = { 'ENABLE_BITCODE' => 'NO', 'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'arm64' }
    spec.user_target_xcconfig = { 'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'arm64' }
  
    spec.vendored_libraries = 'clients/ios/lib/libexocore.a'
    spec.source_files = 'clients/ios/swift/**/*.{swift,h}'
    spec.library      = 'iconv', 'z'

    spec.dependency 'SwiftProtobuf', '~> 1.10'
  end
