import UIKit
import SwiftyJSON
import WebKit

class HybridWebView: AutoLayoutWebView, WKNavigationDelegate {
    fileprivate var callback: ((JSON?) -> Void)!
    fileprivate var ready = false
    fileprivate var component: String!
    fileprivate var nonReadyQueue = [[String: AnyObject]]()

    func initialize(_ component: String, callback: @escaping (JSON?) -> Void) {
        self.navigationDelegate = self

        self.setBackgroundTransparent()

        self.component = component
        self.callback = callback

        let url = URL(fileURLWithPath: Bundle.main.path(forResource: "hybrid", ofType: "html", inDirectory: "js")!)
        self.loadFileURL(url, allowingReadAccessTo: url)
        self.evaluateJavaScript("window.component = \"\(component)\"") { (ret, err) in
            if err != nil {
                print("HybridWebView> Error setting component: \(String(describing: err))")
            }
        }
    }

    func setData(_ data: [String: AnyObject]) {
        if (self.ready) {
            let jsonData = JSON(data)
            let jsonText = (jsonData.rawString()) ?? ""
            self.evaluateJavaScript("window.setData(\(jsonText))") { (ret, err) in
                // nothing to do
            }
            self.invalidateIntrinsicContentSize()
        } else {
            self.nonReadyQueue.append(data)
        }
    }

    func handleReady() {
        self.ready = true;
        for data in self.nonReadyQueue {
            self.setData(data)
        }
        self.nonReadyQueue = []
    }

    // override point
    func handleCallbackData(_ data: JSON) {
        self.callback(data)
    }

    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let url = navigationAction.request.url!

        if (url.scheme == "file" || url.scheme == "about") {
            decisionHandler(.allow)

        } else if (url.scheme == "exomind") {
            self.evaluateJavaScript("window.getData(\(url.host!))") { (data, err) in
                if let msg = data as? String,
                   let msgData = msg.data(using: String.Encoding.utf8, allowLossyConversion: false),
                   let json = try? JSON(data: msgData, options: .allowFragments) {

                    if (json.string == "ready") {
                        self.handleReady()
                        self.checkSize()
                    } else {
                        self.checkSize()
                        self.handleCallbackData(json)
                    }
                } else {
                    print("HybridWebView > Could not parse message from js. Msg \(String(describing: data))")
                }
            }

            decisionHandler(.cancel)
        } else {
            // we open anything else in safari
            UIApplication.shared.open(url, options: [:], completionHandler: { (success) in
                // nothing to do
            })
            decisionHandler(.cancel)
        }
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        self.checkSize()
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        print("HybridWebView > Failed loading page \(error)")
    }

    deinit {
        print("HybridWebView > Deinit")
    }
}
