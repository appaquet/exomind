# iOS client

## Dependencies

**You need to be on MacOS**

* Install Cocoapods
  * `sudo gem install cocoapods`

* Install [Swift Protobuf Plugin](https://github.com/apple/swift-protobuf/)
  * `brew install swift-protobuf`

* Install pods: `pod install`

* Open the project in XCode / AppCode

* Generate a node configuration and copy it to the app:

  `exo config standalone path/to/ios/node/config.yaml --exclude-app-schemas yaml`