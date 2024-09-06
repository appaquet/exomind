# iOS client

## Dependencies

**You need to be on MacOS**

* The iOS and web clients shares some code. Because of this, you need to install [web requirements](../web/README.md) and build for iOS:
  * `cd ../web && yarn build_ios`

* Install Cocoapods
  * `sudo gem install cocoapods`

* Install [Swift Protobuf Plugin](https://github.com/apple/swift-protobuf/)
  * `brew install swift-protobuf`

* Install pods: 
  * `pod install`

* Open the project in XCode / AppCode

## Running
* On first app start, note the discovery code.

* On the main node, add the iOS node with `exo cell node add`, paste the discovery code. Restart your main node.