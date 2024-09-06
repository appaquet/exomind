import Foundation
import KeychainSwift
import Exocore

class ExtensionUtils {
    static func hasKeychainHasEndpoint() -> Bool {
        let keychain = KeychainSwift()
        return keychain.get("store_http_endpoint") != nil && keychain.get("store_auth_token") != nil
    }

    static func setStoreEndpoint(endpoint: String, authToken: String) {
        let keychain = KeychainSwift()
        keychain.set(endpoint, forKey: "store_http_endpoint")
        keychain.set(authToken, forKey: "store_auth_token")
    }

    static func createLinkObject(url: String, title: String, done: (()->Void)? = nil) {
        let keychain = KeychainSwift()
        guard let endpoint = keychain.get("store_http_endpoint"),
              let authToken = keychain.get("store_auth_token") else {
            print("ExtensionUtils> No cookies in keychain")
            return
        }

        var link = Exomind_Base_V1_Link()
        link.url = url
        link.title = title

        var child = Exomind_Base_V1_CollectionChild()
        child.collection.entityID = "inbox"
        child.weight = UInt64(Date().millisecondsSince1970)

        let mutationRequest = try! MutationBuilder
                .createEntity()
                .putTrait(message: link)
                .putTrait(message: child)
                .build()
                .serializedData()

        let url = URL(string: "\(endpoint)store/mutate?token=\(authToken)")!

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/protobuf", forHTTPHeaderField: "Content-Type")
        request.httpBody = mutationRequest

        let task = URLSession.shared.dataTask(with: request as URLRequest, completionHandler: { data, response, error in
            done?()
        })
        task.resume()
    }
}
