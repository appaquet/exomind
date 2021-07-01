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
            // from https://stackoverflow.com/questions/27515236/how-to-determine-the-content-size-of-a-wkwebview
            self.evaluateJavaScript("document.body.scrollHeight", completionHandler: { (height, error) in
                self.maybeChangeHeight(height as! CGFloat)
            })
        }
    }

    private func maybeChangeHeight(_ newHeight: CGFloat) {
        // content is not ready yet. we postpone check
        if newHeight == 0 {
            NSObject.cancelPreviousPerformRequests(withTarget: self)
            self.perform(#selector(checkSize), with: self, afterDelay: 0.1)
            return
        }

        let diff = abs(newHeight - self.height)
        if diff > 1.0 {
            self.height = newHeight

            self.invalidateIntrinsicContentSize()
            self.onHeightChange?(self.height)

            // we check again after delay to make sure height is the same
            NSObject.cancelPreviousPerformRequests(withTarget: self)
            self.perform(#selector(checkSize), with: self, afterDelay: 0.1)
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
