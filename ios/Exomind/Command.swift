//
//  Command.swift
//  Exomind
//
//  Created by Andre-Philippe Paquet on 2015-11-22.
//  Copyright © 2015 Exomind. All rights reserved.
//

import Foundation
import JavaScriptCore

class Command {
    let jsObj: JSValue

    init(jsObj: JSValue) {
        self.jsObj = jsObj
    }

    func onProcessed(_ cb: @escaping (Command?, HCEntity?) -> Void) {
        // register change callback
        let onProcess: @convention(block) (JSValue?, JSValue?) -> Void = { (jsCmd, jsObj) in
            cb(jsCmd.map {
                Command(jsObj: $0)
            }, jsObj.flatMap {
                BridgeEntityConverter.entityFromJavascript($0)
            })
        }
        self.jsObj.setObject(unsafeBitCast(onProcess, to: AnyObject.self), forKeyedSubscript: "onProcessedIosCallback" as (NSCopying & NSObjectProtocol))
        let ref = self.jsObj.forProperty("onProcessedIosCallback")!
        self.jsObj.invokeMethod("onProcessed", withArguments: [ref])
    }
}
