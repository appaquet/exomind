import Foundation
import JavaScriptCore

class Snoozing {
    static func getLaterChoices() -> [LaterTimeChoice] {
        let builderFunc = JSBridge.instance.jsContext.evaluateScript("exomind.dateUtil.getSnoozeChoices")
        let choices = builderFunc?.call(withArguments: [])
        return choices!.toArray().map {
            (choice) -> LaterTimeChoice in
            let dict = choice as! [AnyHashable: Any]
            return LaterTimeChoice(key: dict["key"] as! String, copy: dict["copy"] as! String)
        }
    }

    static func getLaterIcon(_ key: String) -> String {
        JSBridge.instance.jsContext.evaluateScript("exomind.dateUtil.getSnoozeIcon").call(withArguments: [key]).toString()
    }

    static func textDiffToDate(_ textDiff: String) -> Date {
        let builderFunc = JSBridge.instance.jsContext.evaluateScript("exomind.dateUtil.snoozeDate")
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
