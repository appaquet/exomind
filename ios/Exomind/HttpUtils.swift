import Foundation
import KeychainSwift

class HttpUtils {

    static func saveCookies(_ cookies: [HTTPCookie]) {
        for cookie in cookies {
            HTTPCookieStorage.shared.setCookie(cookie)
        }

        HttpUtils.copyCookiesToKeychain()
    }

    static func copyCookiesToKeychain() {
        let nsurl = URL(string: "https://exomind.io")!
        let cookies = HTTPCookieStorage.shared.cookies(for: nsurl)!
        let headers = HTTPCookie.requestHeaderFields(with: cookies)
        let keyChain = KeychainSwift()
        if let cookie = headers["Cookie"] {
            keyChain.set(cookie, forKey: "cookie")
        } else {
            keyChain.delete("cookie")
        }
    }

    static func wipeCookies() {
        let cookieStorage = HTTPCookieStorage.shared
        let cookies = cookieStorage.cookies
        for cookie in cookies! {
            if (cookie.name == "sid" || cookie.name == "bid" || cookie.name == "fid") {
                cookieStorage.deleteCookie(cookie)
            }
        }
        HttpUtils.copyCookiesToKeychain()
    }
}
