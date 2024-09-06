import UIKit
import WebKit

class AutoLayoutWebView: WKWebView {
    var onHeightChange: ((CGFloat) -> Void)?
    var height: CGFloat = 10
    var noDiffCount = 0
    var consecutiveDiffCount = 0

    func setBackgroundTransparent() {
        // default background color to the system color to support dark / light mode
        self.scrollView.backgroundColor = UIColor.systemBackground
        self.backgroundColor = UIColor.systemBackground
        self.isOpaque = false
    }

    @objc func checkSize() {
        if (!self.scrollView.isScrollEnabled) {
            // content is not ready yet. we postpone check
            if self.scrollView.contentSize.height == 0 {
                NSObject.cancelPreviousPerformRequests(withTarget: self)
                self.perform(#selector(checkSize), with: self, afterDelay: 0.005)
                return
            }

            let diff = abs(self.scrollView.contentSize.height - self.height)
            if diff > 1.0 {
                print("AutoLayoutWebView > Height has changed: \(self.height) vs \(self.scrollView.contentSize.height) (diff=\(diff) consecutive=\(self.consecutiveDiffCount))")

                self.consecutiveDiffCount += 1
                self.noDiffCount = 0
                self.height = self.scrollView.contentSize.height

                self.invalidateIntrinsicContentSize()
                self.onHeightChange?(self.height)

                // we check again after delay to make sure height is the same, unless we seem to be in a expanding loop
                NSObject.cancelPreviousPerformRequests(withTarget: self)
                if self.consecutiveDiffCount < 5 {
                    self.perform(#selector(checkSize), with: self, afterDelay: 0.005)
                }
            } else {
                // schedule a check at increasing interval
                self.noDiffCount += 1
                if self.noDiffCount <= 100 {
                    NSObject.cancelPreviousPerformRequests(withTarget: self)
                    self.perform(#selector(checkSize), with: self, afterDelay: Double(self.noDiffCount) / 100.0)
                } else {
                    self.consecutiveDiffCount = 0
                }
            }
        }
    }

    override var intrinsicContentSize: CGSize {
        if (!self.scrollView.isScrollEnabled) {
            self.checkSize()
            return CGSize(width: self.scrollView.contentSize.width, height: self.height)
        } else {
            return super.intrinsicContentSize
        }
    }

    deinit {
        print("AutoLayoutWebView > Deinit")
    }
}
