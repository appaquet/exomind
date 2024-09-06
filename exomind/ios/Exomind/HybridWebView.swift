import UIKit
import SwiftyJSON
import WebKit

class HybridWebView: AutoLayoutWebView, WKNavigationDelegate, WKScriptMessageHandler {
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

        let script = WKUserScript(
                    source: "window.component = \"\(component)\"",
                    injectionTime: WKUserScriptInjectionTime.atDocumentStart,
                    forMainFrameOnly: true
                )
        self.configuration.userContentController.addUserScript(script)
        self.configuration.userContentController.add(MsgHandlerTrampoline(delegate: self), name: "onMessage")
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
            // allow loading blank page & hybrid.js
            decisionHandler(.allow)
            
        } else {
            // we open anything else in safari
            UIApplication.shared.open(url, options: [:], completionHandler: { (success) in
                // nothing to do
            })
            decisionHandler(.cancel)
        }
    }

    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        guard let msg = message.body as? String else { return }
        
        if msg == "ready" {
            self.handleReady()
            self.checkSize()
            return
        }
        
        if let msgData = msg.data(using: String.Encoding.utf8, allowLossyConversion: false),
           let json = try? JSON(data: msgData, options: .allowFragments) {
            self.checkSize()
            self.handleCallbackData(json)
        } else {
            print("HybridWebView > Could not parse message from js. Msg \(String(describing: message))")
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

// Trampoline for message handler to prevent a cycle between the javascript context & the WKWebView
// via the message handler.
//
// See https://stackoverflow.com/questions/26383031/wkwebview-causes-my-view-controller-to-leak/26383032#26383032
class MsgHandlerTrampoline: NSObject, WKScriptMessageHandler {
    weak var delegate : WKScriptMessageHandler?
    
    init(delegate:WKScriptMessageHandler) {
        self.delegate = delegate
        super.init()
    }
    
    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        self.delegate?.userContentController(userContentController, didReceive: message)
    }
}
