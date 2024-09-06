import Foundation
import JavaScriptCore

class JSTimer: NSObject {
    fileprivate let callback: JSValue
    fileprivate let delay: Double
    fileprivate let repeats: Bool
    fileprivate var timer: Timer?

    init(callback: JSValue, delay: Double, repeats: Bool = true) {
        self.callback = callback
        self.delay = delay
        self.repeats = repeats
        super.init()

        // Special case, if it's 1ms and less, it means it's just to dissociate callstack in javascript. Here we go to background thread instead
        if (delay <= 0.001) {
            self.tick()
        } else {
            self.timer = Timer.scheduledTimer(timeInterval: delay, target: self, selector: #selector(tick), userInfo: nil, repeats: repeats)
        }
    }

    @objc func tick() {
        DispatchQueue.main.async {
            self.callback.call(withArguments: [])
        }
    }

    func stop() {
        self.timer?.invalidate()
    }
}
