
import Foundation
import KeychainSwift
import Alamofire

class ExtensionUtils {
    static func hasKeychainCookie() -> Bool {
        let keychain = KeychainSwift()
        return keychain.get("cookie") != nil
    }

    static func createLinkObject(url: String, title: String) {
        let nsurl = URL(string: "https://exomind.io/v1/command")!
        let keychain = KeychainSwift()
        guard let cookie = keychain.get("cookie") else {
            print("ExtensionUtils> No cookie in keychain")
            return
        }

        var request = URLRequest(url: nsurl)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue(cookie, forHTTPHeaderField: "Cookie")

        let isoDateSerializer: DateFormatter = {
            let dateFormatter = DateFormatter()
            dateFormatter.locale = Locale(identifier: "en_US_POSIX")
            dateFormatter.timeZone = TimeZone(abbreviation: "GMT")
            dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS"
            return dateFormatter
        }()
        let now = Date()
        let isoDate = isoDateSerializer.string(from: now) + "Z"
        let weight = now.timeIntervalSince1970 * 1000

        let payload: [String: Any] = [
            "type": "command_entity_traits",
            "addTraits": [
                [
                    "_type": "exomind.link",
                    "title": title,
                    "url": url
                ],
                [
                    "_type": "exomind.child",
                    "date": isoDate,
                    "weight": weight,
                    "to": "inbox"
                ]], "updateTraits": [], "putTraits": [], "removeTraits": []
        ]

        request.httpBody = try! JSONSerialization.data(withJSONObject: payload, options: [])

        Alamofire
            .request(request)
            .responseString { response in
                print(response)
        }
    }

    static func createTaskObject(title: String) {
        let nsurl = URL(string: "https://exomind.io/v1/command")!
        let keychain = KeychainSwift()
        guard let cookie = keychain.get("cookie") else {
            print("ExtensionUtils> No cookie in keychain")
            return
        }

        var request = URLRequest(url: nsurl)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.setValue(cookie, forHTTPHeaderField: "Cookie")

        let isoDateSerializer: DateFormatter = {
            let dateFormatter = DateFormatter()
            dateFormatter.locale = Locale(identifier: "en_US_POSIX")
            dateFormatter.timeZone = TimeZone(abbreviation: "GMT")
            dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS"
            return dateFormatter
        }()
        let now = Date()
        let isoDate = isoDateSerializer.string(from: now) + "Z"
        let weight = now.timeIntervalSince1970 * 1000

        let payload: [String: Any] = [
            "type": "command_entity_traits",
            "addTraits": [
                [
                    "_type": "exomind.task",
                    "title": title,
                ],
                [
                    "_type": "exomind.child",
                    "date": isoDate,
                    "weight": weight,
                    "to": "inbox"
                ]], "updateTraits": [], "putTraits": [], "removeTraits": []
        ]

        request.httpBody = try! JSONSerialization.data(withJSONObject: payload, options: [])

        Alamofire
            .request(request)
            .responseString { response in
                print(response)
        }
    }
}
