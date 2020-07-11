//
//  XMLHttpRequestBridgeExport.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2016-11-03.
//  Copyright © 2016 Exomind. All rights reserved.
//

import Foundation
import JavaScriptCore

protocol XMLHttpRequestBridgeFactory {
    func build() -> XMLHttpRequestBridgeExport
}

@objc protocol XMLHttpRequestBridgeExport: JSExport {
    var onreadystatechange: JSValue? { get set }
    var readyState: Int { get set }
    var status: Int { get set }
    var responseText: String { get set }
    
    // Multiple parameters functions are not supported. Need to use this method instead. Is it safe? Unowned self?
    // https://gist.github.com/zeitiger/1387f7d996f64b493608
    var open: ((_ method:String, _ url:String) -> Void)? { get }
    var send: ((_ data:String) -> Void)? { get }
    var setRequestHeader: ((_ header:String, _ value:String) -> Void)? { get }
}
