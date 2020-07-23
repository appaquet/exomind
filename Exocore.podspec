Pod::Spec.new do |spec|
    spec.name         = 'Exocore'
    spec.version      = '0.1.0'
    spec.license      = { :type => 'Apache-2.0' }
    spec.summary      = 'Distributed applications framework'
    spec.authors      = { 'Andre-Philippe Paquet' => 'appaquet@gmail.com' }
    spec.source       = { :git => 'https://github.com/appaquet/exocore.git', :tag => 'v' +  spec.version.to_s }
    spec.homepage     = 'https://github.com/appaquet/exocore'

    spec.swift_version = '4.2'
    spec.ios.deployment_target = '11.0'
    spec.pod_target_xcconfig = { 'ENABLE_BITCODE' => 'NO' }
  
    spec.vendored_libraries = 'clients/ios/lib/*.a'
    spec.source_files = 'clients/ios/swift/**/*.{swift,h}'
    spec.library      = 'iconv', 'z'

    spec.dependency 'SwiftProtobuf', '~> 1.10'
  end
