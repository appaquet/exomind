import Foundation
import JavaScriptCore

class Emails {
    static func sanitizeHtml(_ html: String) -> String {
        let builderFunc = JSBridge.instance.jsContext.evaluateScript("exomind.emailUtil.sanitizeHtml")
        return builderFunc!.call(withArguments: [html]).toString()
    }

    static func plainTextToHtml(_ text: String) -> String {
        let builderFunc = JSBridge.instance.jsContext.evaluateScript("exomind.emailUtil.plainTextToHtml")
        return builderFunc!.call(withArguments: [text]).toString()
    }

    static func splitOriginalThreadHtml(_ html: String) -> (String, String) {
        let builderFunc = JSBridge.instance.jsContext.evaluateScript("exomind.emailUtil.splitOriginalThreadHtml")
        let rets = builderFunc?.call(withArguments: [html]).toArray()
        return (rets![0] as! String, rets![1] as! String)
    }

    static func formatContact(_ contact: Exomind_Base_V1_Contact) -> String {
        if contact.name != "" {
            return "\(contact.name) <\(contact.email)>"
        } else {
            return contact.email
        }
    }

// TODO:
//    static func injectInlineImages(_ entityTrait: EntityTraitOld, html: String) -> String {
////        let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailUtil.injectInlineImages")
////        if  let jsEntity = BridgeEntityConverter.entityToJavascript(entityTrait.entity),
////            let jsEmail = BridgeEntityConverter.recordToJsRecord(entityTrait.trait) {
////
////            return builderFunc!.call(withArguments: [jsEntity, jsEmail, html]).toString()
////        } else {
////            return html
////        }
//
//        return html
//    }
//
//    static func attachmentUrl(_ entity: HCEntity, email: Email, attachment: FileAttachmentIntegration) -> String? {
//        if let jsEntity = BridgeEntityConverter.entityToJavascript(entity),
//           let jsEmail = BridgeEntityConverter.recordToJsRecord(email),
//           let jsAttachment = BridgeEntityConverter.recordToJsRecord(attachment),
//           let builderFunc = DomainStore.instance.jsContext.evaluateScript("exomind.emailUtil.attachmentUrl") {
//
//            return builderFunc.call(withArguments: [jsEntity, jsEmail, jsAttachment]).toString()
//        } else {
//            return nil
//        }
//    }
}

