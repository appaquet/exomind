import UIKit
import GoogleSignIn
import WebKit

class LoginViewController: UIViewController, GIDSignInUIDelegate {
    @IBOutlet weak var signInButton: GIDSignInButton!

    override func viewDidLoad() {
        let gidSignIn = GIDSignIn.sharedInstance()
        gidSignIn?.uiDelegate = self
        gidSignIn?.signOut()
    }

    @IBAction func googleSignInClick(_ sender: AnyObject) {
        // click on google signin button
    }

    @IBAction func switchBackend(_ sender: AnyObject) {
        let vc = UIAlertController(title: nil, message: nil, preferredStyle: .alert)

        vc.addAction(UIAlertAction(title: "Local", style: .default, handler: { (action) -> Void in
            HttpUtils.wipeCookies()
            HttpUtils.switchBackendType("local")
        }))
        vc.addAction(UIAlertAction(title: "Prod", style: .default, handler: { (action) -> Void in
            HttpUtils.wipeCookies()
            HttpUtils.switchBackendType("prod")
        }))

        vc.addAction(UIAlertAction(title: "Cancel", style: .cancel, handler: nil))
        self.present(vc, animated: true, completion: nil)
    }

    func sign(inWillDispatch signIn: GIDSignIn!, error: Error!) {
        let appDelegate = UIApplication.shared.delegate as? AppDelegate
        appDelegate?.googleSigninCallback = { [weak self] (user) -> Void in
            let vc = GoogleLoginViewController()
            vc.token = user.serverAuthCode
            vc.callback = { (success, cookies, err) in
                print("Login succeed \(success) \(String(describing: err))")
                HttpUtils.saveCookies(cookies)
                JSBridge.instance.resetConnections()
                appDelegate?.googleSigninCallback = nil
            }
            self?.present(vc, animated: true, completion: nil)
        }
    }

    func sign(_ signIn: GIDSignIn!, present viewController: UIViewController!) {
        self.present(viewController, animated: true, completion: nil)
    }

    func sign(_ signIn: GIDSignIn!, dismiss viewController: UIViewController!) {
        self.dismiss(animated: true, completion: nil)
    }
}

class GoogleLoginViewController: UIViewController, WKNavigationDelegate {
    var webview: WKWebView!
    var callback: ((Bool, [HTTPCookie], NSError?) -> Void)?
    var token: String!

    override func viewDidLoad() {
        super.viewDidLoad()

        self.webview = WKWebView()
        self.view.addSubview(self.webview)
        self.webview.frame = self.view.frame
        self.webview.navigationDelegate = self

        var urlComponents = URLComponents(string: "https://exomind.io/v1/auth/google/code")
        urlComponents?.queryItems = [
            URLQueryItem(name: "code", value: token)
        ]

        if let url = urlComponents?.url {
            let req = URLRequest(url: url)
            self.webview.load(req)
        } else {
            self.callback?(false, [], nil)
        }
    }

    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let request = navigationAction.request
        if (request.url?.path == "/") {
            let cookieStore = self.webview.configuration.websiteDataStore.httpCookieStore
            cookieStore.getAllCookies { [weak self] (cookies) in
                self?.callback?(true, cookies, nil)
            }
            self.dismiss(animated: true, completion: nil)
            decisionHandler(.cancel)
        } else {
            decisionHandler(.allow)
        }
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        self.callback?(false, [], error as NSError?)
        self.dismiss(animated: true, completion: nil)
    }
}
