import Foundation

typealias Handler = () -> Void

// Adapted from https://bradfol.com/how-can-i-debounce-a-method-call-in-swift-4/a
class Debouncer {
    private let timeInterval: TimeInterval
    private var timer: Timer?

    var handler: Handler?

    init(timeInterval: TimeInterval, handler: Handler?) {
        self.timeInterval = timeInterval
        self.handler = handler
    }

    func renewInterval() {
        timer?.invalidate()
        timer = Timer.scheduledTimer(withTimeInterval: timeInterval, repeats: false, block: { [weak self] timer in
            self?.handleTimer(timer)
        })
    }

    private func handleTimer(_ timer: Timer) {
        guard timer.isValid else {
            return
        }

        handler?()
    }
}