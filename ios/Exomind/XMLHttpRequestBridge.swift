import Foundation

import Foundation
import JavaScriptCore
import Alamofire

class RealXMLHttpRequestBridgeFactory: XMLHttpRequestBridgeFactory {
    func build() -> XMLHttpRequestBridgeExport {
        XMLHttpRequestBridge()
    }
}

class XMLHttpRequestBridge: NSObject, XMLHttpRequestBridgeExport {
    dynamic var onreadystatechange: JSValue?
    dynamic var readyState: Int = 0
    dynamic var status: Int = 0
    dynamic var responseText: String = ""
    dynamic var open: ((_ method: String, _ url: String) -> Void)?
    dynamic var send: ((_ data: String) -> Void)?
    dynamic var setRequestHeader: ((_ header: String, _ value: String) -> Void)?
    var url: String?
    var method: String = "POST"

    override init() {
        super.init()
        self.open = self.openImpl
        self.send = self.sendImpl
        self.setRequestHeader = self.setRequestHeaderImpl
    }

    func openImpl(_ method: String, url: String) -> Void {
        self.url = url
        self.method = method
    }

    func setRequestHeaderImpl(_ header: String, value: String) {
        // TODO: Support it?
    }

    func sendImpl(_ data: String) {
        print("XMLHttpRequestBridge > Sending(\(self.method), \(String(describing: self.url)), \(data))")
        var request = URLRequest(url: URL(string: self.url!)!)
        request.httpMethod = self.method

        if (self.method == "POST" || self.method == "PUT") {
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
            request.httpBody = data.data(using: String.Encoding.utf8)
        }

        Alamofire
                .request(request)
                .responseString(encoding: String.Encoding.utf8) { response in
                    if let err = response.result.error {
                        self.status = response.response?.statusCode ?? 533
                        print("XMLHttpRequestBridge > Got error status=\(self.status) err=\(err)")
                        self.readyState = 4
                        self.responseText = err.localizedDescription
                        let _ = self.onreadystatechange?.call(withArguments: [])
                    } else {
                        self.status = response.response?.statusCode ?? 200
                        print("XMLHttpRequestBridge > Got data status=\(self.status)")
                        self.readyState = 4
                        self.responseText = response.result.value ?? ""
                        let _ = self.onreadystatechange?.call(withArguments: [])
                    }

                    // helps cleanup memory
                    self.onreadystatechange = nil
                    self.open = nil
                    self.send = nil
                    self.setRequestHeader = nil
                }
    }

}
