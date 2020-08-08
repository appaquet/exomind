import Foundation

class Retry {
    private let minimumInterval: TimeInterval
    private var timer: Timer?
    private var lastAttempt: Date?

    var handler: Handler?

    init(minimumInterval: TimeInterval, handler: Handler?) {
        self.minimumInterval = minimumInterval
        self.handler = handler
    }

    func trigger() {
        // TODO: Exponential
        let now = Date()
        if let lastAttempt = lastAttempt {
            let interval = now - lastAttempt
            if interval < self.minimumInterval {
                if self.timer == nil {
                    self.timer = Timer.scheduledTimer(withTimeInterval: interval, repeats: false, block: { [weak self] timer in
                        self?.trigger()
                    })
                }

                return
            }
        }

        self.handler?()
        self.lastAttempt = Date()
        self.timer?.invalidate()
        self.timer = nil
    }
}
