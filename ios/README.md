# iOS client

## Dependencies

**You need to be on MacOS**

* Install Cocoapods
  * `sudo gem install cocoapods`

* Install [Swift Protobuf Plugin](https://github.com/apple/swift-protobuf/)
  * `brew install swift-protobuf`

## Known issues
* App gets terminated due to signal 13 when resuming from background.
  * Mio doesn't handle `SIGPIPE` signal when Tokio tries to write to a closed connection. See (here)[https://github.com/tokio-rs/mio/issues/949].
  * You can ignore the the signal by adding this in your app (ex: in `didFinishLaunchingWithOptions`):
    ```swift
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        signal(SIGPIPE, SIG_IGN)
        ...
        return true
    }
    ```