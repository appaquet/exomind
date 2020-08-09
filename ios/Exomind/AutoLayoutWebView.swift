import UIKit
import WebKit

class AutoLayoutWebView: WKWebView {
    var onHeightChange: ((CGFloat) -> Void)?
    var height: CGFloat = 0

    func setBackgroundTransparent() {
        // default background color to the system color to support dark / light mode
        self.scrollView.backgroundColor = UIColor.systemBackground
        self.backgroundColor = UIColor.systemBackground
        self.isOpaque = false
    }

    @objc func checkSize() {
        if (!self.scrollView.isScrollEnabled) {
            let diff = abs(self.scrollView.contentSize.height - self.height)
            if diff > 1.0 {
                self.height = self.scrollView.contentSize.height

                self.invalidateIntrinsicContentSize()
                self.onHeightChange?(self.height)

                NSObject.cancelPreviousPerformRequests(withTarget: self)
                self.perform(#selector(checkSize), with: self, afterDelay: 0.1)
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
