Pod::Spec.new do |spec|
    spec.name         = 'Exocore'
    spec.version      = '0.1.27'
    spec.license      = { :type => 'Apache-2.0' }
    spec.summary      = 'Distributed applications framework'
    spec.authors      = { 'Andre-Philippe Paquet' => 'appaquet@gmail.com' }
    spec.source       = { :http => 'https://github.com/appaquet/exocore/releases/download/v' + spec.version.to_s + '/exocore-ios.tar.gz', :type => 'tgz' }
    spec.homepage     = 'https://github.com/appaquet/exocore'

    spec.swift_version = '5.0'
    spec.ios.deployment_target = '17.0'

    # Rust binaries don't have bitcode (would need to build with https://github.com/getditto/rust-bitcode)
    spec.pod_target_xcconfig = { 'ENABLE_BITCODE' => 'NO' }

    spec.vendored_frameworks = 'clients/ios/lib/ExocoreLibs.xcframework'
    spec.source_files = 'clients/ios/swift/**/*.{swift,h}'
    spec.library      = 'iconv', 'z'

    spec.dependency 'SwiftProtobuf', '~> 1.10'

    spec.frameworks = 'SystemConfiguration' # referenced by the if-watch crate used in libp2p
  end
