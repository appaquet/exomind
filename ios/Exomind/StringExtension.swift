
import Foundation

extension String  {
    // from http://stackoverflow.com/questions/24123518/how-to-use-cc-md5-method-in-swift-language
    var md5: String! {
        get {
            let length = Int(CC_MD5_DIGEST_LENGTH)
            var digest = [UInt8](repeating: 0, count: length)

            if let d = self.data(using: String.Encoding.utf8) {
                let _ = d.withUnsafeBytes { (body: UnsafePointer<UInt8>) in
                    CC_MD5(body, CC_LONG(d.count), &digest)
                }
            }

            return (0..<length).reduce("") {
                $0 + String(format: "%02x", digest[$1])
            }
        }
    }

    func nonEmpty() -> String? {
        if !self.isEmpty {
            return self
        }  else {
            return nil
        }
    }
}
