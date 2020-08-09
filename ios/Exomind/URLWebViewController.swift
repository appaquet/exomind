import UIKit
import SnapKit
import WebKit

class URLWebViewController: UIViewController, WKNavigationDelegate {
    var url: URL?
    var webview: WKWebView!

    convenience init(url: URL) {
        self.init()
        self.url = url
    }

    override func viewDidLoad() {
        self.webview = WKWebView()
        self.webview.navigationDelegate = self

        self.view.addSubview(webview)
        self.webview.snp.makeConstraints { (make) in
            make.center.equalTo(self.view.snp.center)
            make.width.equalTo(self.view.snp.width)
            make.height.equalTo(self.view.snp.height)
        }
        self.webview.load(URLRequest(url: self.url!))
    }

    override func viewWillAppear(_ animated: Bool) {
        (self.navigationController as? NavigationController)?.resetState()
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        let contentSize = webView.scrollView.contentSize
        let viewSize = webView.bounds.size
        let rw = viewSize.width / contentSize.width
        webView.scrollView.minimumZoomScale = 0.1
        webView.scrollView.maximumZoomScale = 2.0
        webView.scrollView.zoomScale = rw
    }
}
