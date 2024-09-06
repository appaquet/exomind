import UIKit
import WebKit

class EmailBodyWebView: AutoLayoutWebView, WKNavigationDelegate {
    var onLinkClick: ((URL) -> Bool)?
    var onLoaded: (() -> Void)?

    func initialize() {
        self.navigationDelegate = self
        self.scrollView.isScrollEnabled = false
        self.scrollView.bouncesZoom = false
        self.setBackgroundTransparent()
    }

    func loadEmailEntity(_ parts: [Exomind_Base_V1_EmailPart], short: Bool = true) {
        for part in parts {
            if part.mimeType == "text/html" {
                var content: String!
                if (short) {
                    let (cnt, _) = Emails.splitOriginalThreadHtml(part.body)
                    content = cnt
                } else {
                    content = part.body
                }

                let inlinedAttachmentBody = content // TODO: EmailsLogic.injectInlineImages(entityTrait, html: content)
                let sanitized = Emails.sanitizeHtml(inlinedAttachmentBody!)
                self.loadHTML(sanitized)

                // if it's html, we break here since plain may be present after
                break
            } else {
                let htmlbody = Emails.plainTextToHtml(part.body)
                self.loadHTML(htmlbody)
            }
        }
    }

    func loadHTML(_ body: String) {
        // http://stackoverflow.com/questions/9993393/how-to-toggle-uiwebview-text-when-scalespagetofit-no
        let head = "<meta http-equiv='Content-Type' content='text/html; charset=utf-8'><meta name='viewport' id='iphone-viewport' content='width=device-width'>"

        // Triggered when all DOM elements are ready (but not necessarily images)
        let ready = "<script>document.addEventListener(\"DOMContentLoaded\", function(event) { window.location = \"exomind://ready/\" + document.body.clientHeight; });</script>"

        // Triggered when the page is fully loaded, with all images
        let loaded = "window.location = 'exomind://loaded/' + document.body.clientHeight"

        // For Dark Mode. See web's email-thread.less
        let style = "<style>@media (prefers-color-scheme: dark) { body { filter: invert(1) hue-rotate(180deg); } img, div[style*=\"background-image\"] { filter: invert(1) hue-rotate(180deg); } }</style>"
        let final = "<html><head>\(head)\(style)</head><body style=\"width: 100%; padding: 10px 5px;\" onload=\"javascript: \(loaded)\">\(body)\(ready)</body></html>"

        self.loadHTMLString(final, baseURL: nil)
    }

    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let url = navigationAction.request.url!

        // we don't want any website to open inside the frame, we open in safari
        if (url.scheme == "about") {
            decisionHandler(.allow)

        } else if (url.scheme == "exomind" && url.host == "ready") {
            self.checkSize()
            self.onLoaded?()
            decisionHandler(.cancel)

        } else if (url.scheme == "exomind" && url.host == "loaded") {
            self.checkSize()
            decisionHandler(.cancel)

        } else {
            let shouldAllow = self.onLinkClick?(url) ?? true
            if shouldAllow {
                decisionHandler(.allow)
            } else {
                decisionHandler(.cancel)
            }
        }
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        self.checkSize()
    }

    deinit {
        print("EmailBodyWebView > Deinit")
    }

}
