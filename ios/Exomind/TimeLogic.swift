import Foundation
import JavaScriptCore

class TimeLogic {

    static func getLaterChoices() -> [LaterTimeChoice] {
        let builderFunc = JSBridge.instance.jsContext.evaluateScript("exomind.timeLogic.getLaterChoices")
        let choices = builderFunc?.call(withArguments: [])
        return choices!.toArray().map {
            (choice) -> LaterTimeChoice in
            let dict = choice as! [AnyHashable: Any]
            return LaterTimeChoice(key: dict["key"] as! String, copy: dict["copy"] as! String)
        }
    }

    static func getLaterIcon(_ key: String) -> String {
        return JSBridge.instance.jsContext.evaluateScript("exomind.timeLogic.getLaterIcon").call(withArguments: [key]).toString()
    }

    static func textDiffToDate(_ textDiff: String) -> Date {
        let builderFunc = JSBridge.instance.jsContext.evaluateScript("exomind.timeLogic.textDiffToDate")
        let jsDate = builderFunc?.call(withArguments: [textDiff])
        return jsDate!.toDate()
    }
}

class LaterTimeChoice {
    var key: String
    var copy: String

    init(key: String, copy: String) {
        self.key = key
        self.copy = copy
    }
}
