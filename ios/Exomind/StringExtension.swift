//
// Created by Andre-Philippe Paquet on 2016-08-07.
// Copyright (c) 2016 Exomind. All rights reserved.
//

import Foundation

extension String  {
    // from http://stackoverflow.com/questions/24123518/how-to-use-cc-md5-method-in-swift-language
    var md5: String! {
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
