//
//  EmailBodyWebView.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-03-07.
//  Copyright © 2016 Exomind. All rights reserved.
//

import UIKit
import WebKit

class EmailBodyWebView: AutoLayoutWebView, WKNavigationDelegate {
    var entityTrait: EntityTraitOld?
    var onLinkClick: ((URL) -> Bool)?

    func initialize() {
        self.navigationDelegate = self
        self.scrollView.isScrollEnabled = false
        self.scrollView.bouncesZoom = false
    }
    
    func loadEmailEntity(_ entityTrait: EntityTraitOld, short: Bool = true) {
        self.entityTrait = entityTrait
        var parts: [EmailPart]?
        if let email = entityTrait.trait as? EmailFull {
            parts = email.parts
        } else if let draft = entityTrait.trait as? DraftEmailFull {
            parts = draft.parts
        }
        
        if let parts = parts {
            for part in parts {
                if let htmlPart = part as? EmailPartHtmlFull {
                    let body = htmlPart.body
                    
                    var content: String!
                    if (short) {
                        let (cnt, _) = EmailsLogic.splitOriginalThreadHtml(body)
                        content = cnt
                    } else {
                        content = body
                    }
                    
                    let inlinedAttachmentBody = EmailsLogic.injectInlineImages(entityTrait, html: content)
                    let sanitized = EmailsLogic.sanitizeHtml(inlinedAttachmentBody)
                    self.loadHTML(sanitized)
                    
                    // if it's html, we break here since plain may be present after
                    break
                } else if let plainPart = part as? EmailPartPlainFull {
                    let htmlbody = EmailsLogic.plainTextToHtml(plainPart.body)
                    self.loadHTML(htmlbody)
                }
            }
        } else {
            self.loadHTML("Loading...")
        }
    }

    func loadHTML(_ body: String) {
        // http://stackoverflow.com/questions/9993393/how-to-toggle-uiwebview-text-when-scalespagetofit-no
        let head = "<meta http-equiv='Content-Type' content='text/html; charset=utf-8'><meta name='viewport' id='iphone-viewport' content='width=device-width'>"
        let ready = "<script>document.addEventListener(\"DOMContentLoaded\", function(event) { window.location = \"exomind://loaded/\" + document.body.clientHeight; });</script>"

        // For Dark Mode. Sync with Web's html-editor.js
        let style = "<style>@media (prefers-color-scheme: dark) { body { color: white; background-color: black } a { color: #4285f4 } }</style>"
        let final = "<html><head>\(head)\(style)</head><body style=\"width: 100%; padding: 10px 5px;\">\(body)\(ready)</body></html>"
        self.loadHTMLString(final, baseURL: nil)
    }

    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let url = navigationAction.request.url!

        // we don't want any website to open inside the frame, we open in safari
        if (url.scheme == "about") {
            decisionHandler(.allow)
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
